use std::marker::PhantomData;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_2d::Transparent2d,
        core_3d::{AlphaMask3d, Opaque3d, Transparent3d},
    },
    prelude::*,
    reflect::{GetTypeRegistration, TypeUuid},
    render::{
        render_phase::AddRenderCommand,
        render_resource::{Buffer, ShaderRef},
        view::RenderLayers,
        Extract, RenderApp, RenderSet,
    },
    utils::FloatOrd,
};
use bitfield::bitfield;
use bytemuck::Pod;
use wgpu::VertexAttribute;

use crate::{painter::ShapeInstance, prelude::*, ShapePipelineType};

pub(crate) mod pipeline;
use pipeline::*;

pub(crate) mod commands;
use commands::*;

pub(crate) mod render_2d;
use render_2d::*;

pub(crate) mod render_3d;
use render_3d::*;

/// Handler to shader containing shared functionality.
pub const BINDINGS_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13215291696265391738);

/// Handler to shader containing shared functionality.
pub const FUNCTIONS_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 14523762397345674763);

/// Handler to shader for drawing discs.
pub const DISC_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 12563478638216678166);

/// Handler to shader for drawing lines.
pub const LINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13656934768948239208);

/// Handler to shader for drawing regular polygons.
pub const NGON_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17394960287230910395);

/// Handler to shader for drawing rectangles.
pub const RECT_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 15069348348279052351);

/// Load the libraries shaders as internal assets.
pub fn load_shaders(app: &mut App) {
    load_internal_asset!(
        app,
        BINDINGS_HANDLE,
        "shaders/bindings.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        FUNCTIONS_HANDLE,
        "shaders/functions.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        DISC_HANDLE,
        "shaders/shapes/disc.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        LINE_HANDLE,
        "shaders/shapes/line.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        NGON_HANDLE,
        "shaders/shapes/ngon.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        RECT_HANDLE,
        "shaders/shapes/rect.wgsl",
        Shader::from_wgsl
    );
}

/// Collection of shape data in pairs of [`ShapePipelineMaterial`] and [`ShapeData`].
#[derive(Component, Deref, DerefMut)]
pub struct ShapeInstances<T: ShapeData>(pub Vec<ShapeInstance<T>>);

/// Trait implemented by each shapes shader data, defines common methods used in the rendering pipeline.
pub trait ShapeData: Send + Sync + Pod {
    /// Corresponding component representing the given shape.
    type Component: ShapeComponent<Data = Self>;
    /// Vertex layout to be sent to the shader.
    fn vertex_layout() -> Vec<VertexAttribute>;
    /// Reference to the shader to be used when rendering the shape.
    fn shader() -> ShaderRef;
    /// Distance to the shape to be used for z-ordering in 2D.
    fn distance(&self) -> f32 {
        self.transform().transform_point3(Vec3::ZERO).z
    }
    /// Transform of the shape to be used for z-ordering in 3D.
    fn transform(&self) -> Mat4;
}

/// Trait implemented by the corresponding component for each shape type.
pub trait ShapeComponent: Component + GetTypeRegistration {
    type Data: ShapeData<Component = Self>;
    fn into_data(&self, tf: &GlobalTransform) -> Self::Data;
}

/// Marker component to determine shape type for [`ShapeDataBuffer`] entities.
#[derive(Component)]
pub struct ShapeType<T: ShapeData> {
    _marker: PhantomData<T>,
}

impl<T: ShapeData> Default for ShapeType<T> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

/// Buffer of instances for a given shape type determined by [`ShapeType`].
#[derive(Component)]
pub struct ShapeDataBuffer {
    view: Entity,
    material: ShapePipelineMaterial,
    buffer: Buffer,
    distance: f32,
    length: usize,
}

bitfield! {
    /// Flags consumed in shape shaders
    pub struct Flags(u32);
    pub u32, from into ThicknessType, _, set_thickness_type: 1, 0;
    pub u32, from into Alignment, _, set_alignment: 2, 2;
    pub u32, _, set_hollow: 3, 3;
    pub u32, from into Cap, _, set_cap: 5, 4;
    pub u32, _, set_arc: 6, 6;
}

/// Properties attached to a batch of shapes that are needed for pipeline specialization
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ShapePipelineMaterial {
    render_layers: RenderLayers,
    alpha_mode: AlphaModeOrd,
    disable_laa: bool,
    texture: Option<Handle<Image>>,
    canvas: Option<Entity>,
    pipeline: ShapePipelineType,
}

impl ShapePipelineMaterial {
    pub fn new(material: Option<&ShapeMaterial>, render_layers: Option<&RenderLayers>) -> Self {
        let material = material.cloned().unwrap_or_default();
        Self {
            render_layers: render_layers.cloned().unwrap_or_default(),
            alpha_mode: AlphaModeOrd(material.alpha_mode),
            disable_laa: material.disable_laa || material.alpha_mode == AlphaMode::Opaque,
            canvas: material.canvas,
            pipeline: material.pipeline,
            texture: material.texture,
        }
    }
}

impl From<&ShapeConfig> for ShapePipelineMaterial {
    fn from(config: &ShapeConfig) -> Self {
        Self {
            render_layers: config.render_layers.unwrap_or_default(),
            alpha_mode: AlphaModeOrd(config.alpha_mode),
            disable_laa: config.disable_laa || config.alpha_mode == AlphaMode::Opaque,
            texture: config.texture.clone(),
            pipeline: config.pipeline,
            canvas: config.canvas,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AlphaModeOrd(AlphaMode);

impl AlphaModeOrd {
    fn ord(&self) -> f32 {
        match self.0 {
            AlphaMode::Opaque => 0.0,
            AlphaMode::Blend => 1.0,
            AlphaMode::Premultiplied => 3.0,
            AlphaMode::Add => 4.0,
            AlphaMode::Multiply => 5.0,
            AlphaMode::Mask(m) => 6.0 + m,
        }
    }
}

impl PartialOrd for AlphaModeOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ord().partial_cmp(&other.ord())
    }
}

impl Ord for AlphaModeOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.ord()).cmp(&FloatOrd(other.ord()))
    }
}

/// System that extracts [`RenderLayers`] for each camera
///
/// Having to do this isn't ideal but with the way the render pipeline is setup for shapes using `visible_entities` is not ideal either.
/// This may be removed once a better implementation is possible.
pub fn extract_render_layers(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &RenderLayers), With<Camera>>>,
) {
    for (entity, render_layers) in &cameras {
        commands.get_or_spawn(entity).insert(*render_layers);
    }
}

fn setup_pipeline(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<ShapePipelines>()
        .init_resource::<ShapeTextureBindGroups>()
        .add_system(extract_render_layers.in_schedule(ExtractSchedule))
        .add_system(queue_shape_view_bind_groups.in_set(RenderSet::Queue))
        .add_system(queue_shape_texture_bind_groups.in_set(RenderSet::Queue));
}

fn setup_pipeline_3d(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, DrawShapeCommand>()
        .add_render_command::<Transparent3d, DrawShapeCommand>()
        .add_render_command::<AlphaMask3d, DrawShapeCommand>();
}

fn setup_pipeline_2d(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_render_command::<Transparent2d, DrawShapeCommand>();
}

fn setup_type_pipeline<T: ShapeData>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<ShapePipeline<T>>();
}

fn setup_type_pipeline_3d<T: ShapeData>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_system(extract_shapes_3d::<T>.in_schedule(ExtractSchedule))
        .add_system(prepare_shape_buffers_3d::<T>.in_set(RenderSet::Prepare))
        .add_system(queue_shapes_3d::<T>.in_set(RenderSet::Queue));
}

fn setup_type_pipeline_2d<T: ShapeData>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_system(extract_shapes_2d::<T>.in_schedule(ExtractSchedule))
        .add_system(prepare_shape_buffers_2d::<T>.in_set(RenderSet::Prepare))
        .add_system(queue_shapes_2d::<T>.in_set(RenderSet::Queue));
}

/// Plugin that sets up the 2d render pipeline for the given [`ShapeComponent`].
#[derive(Default)]
pub struct ShapeTypePlugin<T: ShapeComponent>(PhantomData<T>);

impl<T: ShapeComponent> Plugin for ShapeTypePlugin<T> {
    fn build(&self, app: &mut App) {
        app.register_type::<T>();
        setup_type_pipeline::<T::Data>(app);
        setup_type_pipeline_2d::<T::Data>(app);
    }
}

/// Plugin that sets up the 3d render pipeline for the given [`ShapeComponent`].
///
/// Requires [`ShapeTypePlugin`] of the same type to have already been built.
#[derive(Default)]
pub struct ShapeType3dPlugin<T: ShapeComponent>(PhantomData<T>);

impl<T: ShapeComponent> Plugin for ShapeType3dPlugin<T> {
    fn build(&self, app: &mut App) {
        setup_type_pipeline_3d::<T::Data>(app);
    }
}

/// Plugin that sets up shared components for [`ShapeTypePlugin`].
pub struct ShapeRenderPlugin;

impl Plugin for ShapeRenderPlugin {
    fn build(&self, app: &mut App) {
        load_shaders(app);
        setup_pipeline(app);
        setup_pipeline_2d(app);
    }
}

/// Plugin that sets up shared components for [`ShapeType3dPlugin`].
pub struct Shape3dRenderPlugin;

impl Plugin for Shape3dRenderPlugin {
    fn build(&self, app: &mut App) {
        setup_pipeline_3d(app);
    }
}
