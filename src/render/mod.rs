use std::marker::PhantomData;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_2d::Transparent2d,
        core_3d::{AlphaMask3d, Opaque3d, Transparent3d},
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::{ComponentUniforms, UniformComponentPlugin},
        render_phase::AddRenderCommand,
        render_resource::{
            encase::private::WriteInto, Buffer, ShaderType, SpecializedRenderPipelines,
        },
        view::RenderLayers,
        Extract, RenderApp, RenderSet,
    },
    utils::FloatOrd,
};
use bitfield::bitfield;
use bytemuck::Pod;
use wgpu::VertexAttribute;

use crate::prelude::*;

pub(crate) mod pipeline;
use pipeline::*;

pub(crate) mod commands;
use commands::*;

pub(crate) mod instanced_2d;
use instanced_2d::*;

pub(crate) mod instanced_3d;
use instanced_3d::*;

pub(crate) mod batched_pipeline;
use batched_pipeline::*;

/// Handler to shader containing shared functionality.
pub const CORE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13215291696265391738);

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
    load_internal_asset!(app, CORE_HANDLE, "shaders/core.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, DISC_HANDLE, "shaders/disc.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, LINE_HANDLE, "shaders/line.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, NGON_HANDLE, "shaders/ngon.wgsl", Shader::from_wgsl);
    load_internal_asset!(app, RECT_HANDLE, "shaders/rect.wgsl", Shader::from_wgsl);
}

/// Collection of instances extracted from components into pairs of [`RenderKey`] and [`Instanceable`].
#[derive(Component, Deref, DerefMut)]
pub struct InstanceData<T>(pub Vec<(RenderKey, T)>);

/// Trait implemented by each type of shape, defines common methods used in the rendering pipeline for instancing.
pub trait Instanceable: Component + ShaderType + Clone + WriteInto + Pod {
    type Component: InstanceComponent<Self>;
    fn vertex_layout() -> Vec<VertexAttribute>;
    fn shader() -> Handle<Shader>;
    fn distance(&self) -> f32;
    fn transform(&self) -> Mat4;
    fn null_instance() -> Self;
}

/// Trait implemented by the corresponding component for each shape type.
pub trait InstanceComponent<T: Instanceable>: Component {
    fn instance(&self, tf: &GlobalTransform) -> T;
}

/// Buffer of instances for a given shape type.
#[derive(Component)]
pub struct InstanceBuffer<T: Instanceable> {
    view: Entity,
    key: RenderKey,
    buffer: Buffer,
    distance: f32,
    length: usize,
    _marker: PhantomData<T>,
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
#[derive(Component, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RenderKey {
    render_layers: RenderLayers,
    alpha_mode: AlphaModeOrd,
    disable_laa: bool,
}

impl RenderKey {
    pub const PLACE_HOLDER: Self = Self {
        render_layers: RenderLayers::none(),
        alpha_mode: AlphaModeOrd(AlphaMode::Add),
        disable_laa: false,
    };

    pub fn new(flags: Option<&Shape>, render_layers: Option<&RenderLayers>) -> Self {
        let flags = flags.cloned().unwrap_or_default();
        Self {
            render_layers: render_layers.cloned().unwrap_or_default(),
            alpha_mode: AlphaModeOrd(flags.alpha_mode),
            disable_laa: flags.disable_laa || flags.alpha_mode == AlphaMode::Opaque,
        }
    }
}

impl From<&ShapeConfig> for RenderKey {
    fn from(config: &ShapeConfig) -> Self {
        Self {
            render_layers: config.render_layers.unwrap_or_default(),
            alpha_mode: AlphaModeOrd(config.alpha_mode),
            disable_laa: config.disable_laa || config.alpha_mode == AlphaMode::Opaque,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
/// This may be removed once a better implementation is complete.
pub fn extract_render_layers(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &RenderLayers), With<Camera>>>,
) {
    for (entity, render_layers) in &cameras {
        commands.get_or_spawn(entity).insert(*render_layers);
    }
}

/// Sets up the pipeline for the specified instanceable shape in the given app;
pub fn setup_instanced_pipeline<T: Instanceable>(app: &mut App) {
    app.add_event::<ShapeEvent<T>>();
    app.sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, DrawInstancedCommand<T>>()
        .add_render_command::<Transparent3d, DrawInstancedCommand<T>>()
        .add_render_command::<AlphaMask3d, DrawInstancedCommand<T>>()
        .init_resource::<InstancedPipeline<T>>()
        .init_resource::<SpecializedRenderPipelines<InstancedPipeline<T>>>()
        .add_system(extract_instances::<T>.in_schedule(ExtractSchedule))
        .add_system(extract_render_layers.in_schedule(ExtractSchedule))
        .add_system(prepare_instance_buffers::<T>.in_set(RenderSet::Prepare))
        .add_system(queue_instances::<T>.in_set(RenderSet::Queue))
        .add_system(queue_instance_view_bind_groups::<T>.in_set(RenderSet::Queue));
}

/// Sets up the pipeline for the specified instanceable shape in the given app;
pub fn setup_instanced_pipeline_2d<T: Instanceable>(app: &mut App) {
    app.add_event::<ShapeEvent<T>>();
    app.sub_app_mut(RenderApp)
        .add_render_command::<Transparent2d, DrawShape<T>>()
        .init_resource::<ShapeMeta<T>>()
        .init_resource::<ExtractedShapes<T>>()
        .init_resource::<ShapePipeline<T>>()
        .init_resource::<SpecializedRenderPipelines<ShapePipeline<T>>>()
        .add_system(extract_shapes::<T>.in_schedule(ExtractSchedule))
        .add_system(extract_render_layers.in_schedule(ExtractSchedule))
        .add_system(prepare_instance_buffers_2d::<T>.in_set(RenderSet::Prepare))
        .add_system(queue_shapes::<T>.in_set(RenderSet::Queue));
}
