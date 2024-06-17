use std::hash::Hasher;
use std::marker::PhantomData;
use std::{hash::Hash, ops::Deref};

use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_2d::Transparent2d,
        core_3d::{AlphaMask3d, Opaque3d, Transparent3d},
    },
    ecs::entity::EntityHashMap,
    prelude::*,
    reflect::GetTypeRegistration,
    render::{
        render_phase::{
            AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, RenderPhase,
        },
        render_resource::{
            Buffer, CachedRenderPipelineId, GpuArrayBuffer, GpuArrayBufferable, ShaderDefVal,
            ShaderRef,
        },
        renderer::{RenderDevice, RenderQueue},
        view::RenderLayers,
        Extract, Render, RenderApp, RenderSet,
    },
    utils::{nonmax::NonMaxU32, FloatOrd},
};
use bitfield::bitfield;
use wgpu::{util::BufferInitDescriptor, BufferUsages, VertexAttribute};

use crate::prelude::*;

pub(crate) mod pipeline;
use pipeline::*;

pub(crate) mod commands;
use commands::*;

pub(crate) mod render_2d;
use render_2d::*;

pub(crate) mod render_3d;
use render_3d::*;

/// Handler to shader containing shared functionality.
pub const CORE_HANDLE: Handle<Shader> = Handle::weak_from_u128(13215291696265391738);

/// Handler to shader containing shared constants.
pub const CONSTANTS_HANDLE: Handle<Shader> = Handle::weak_from_u128(14523762397345674763);

/// Handler to shader for drawing discs.
pub const DISC_HANDLE: Handle<Shader> = Handle::weak_from_u128(12563478638216678166);

/// Handler to shader for drawing lines.
pub const LINE_HANDLE: Handle<Shader> = Handle::weak_from_u128(13656934768948239208);

/// Handler to shader for drawing regular polygons.
pub const NGON_HANDLE: Handle<Shader> = Handle::weak_from_u128(17394960287230910395);

/// Handler to shader for drawing rectangles.
pub const RECT_HANDLE: Handle<Shader> = Handle::weak_from_u128(15069348348279052351);

/// Handler to shader for drawing triangles.
pub const TRIANGLE_HANDLE: Handle<Shader> = Handle::weak_from_u128(12344032791831516511);

/// Load the libraries shaders as internal assets.
pub fn load_shaders(app: &mut App) {
    load_internal_asset!(app, CORE_HANDLE, "shaders/core.wgsl", Shader::from_wgsl);
    load_internal_asset!(
        app,
        CONSTANTS_HANDLE,
        "shaders/constants.wgsl",
        Shader::from_wgsl
    );
    let defs = DiscData::shader_defs(app);
    load_internal_asset!(
        app,
        DISC_HANDLE,
        "shaders/shapes/disc.wgsl",
        Shader::from_wgsl_with_defs,
        defs
    );
    let defs = NgonData::shader_defs(app);
    load_internal_asset!(
        app,
        LINE_HANDLE,
        "shaders/shapes/line.wgsl",
        Shader::from_wgsl_with_defs,
        defs
    );
    let defs = NgonData::shader_defs(app);
    load_internal_asset!(
        app,
        NGON_HANDLE,
        "shaders/shapes/ngon.wgsl",
        Shader::from_wgsl_with_defs,
        defs
    );
    let defs = RectData::shader_defs(app);
    load_internal_asset!(
        app,
        RECT_HANDLE,
        "shaders/shapes/rect.wgsl",
        Shader::from_wgsl_with_defs,
        defs
    );
    load_internal_asset!(
        app,
        TRIANGLE_HANDLE,
        "shaders/shapes/tri.wgsl",
        Shader::from_wgsl
    );
}

/// Contains data necessary to render a single shape.
#[derive(Clone)]
pub struct ShapeInstance<T> {
    /// This shape's material.
    pub material: ShapePipelineMaterial,

    /// The point in space used for ordering this point.
    /// Ignored by the 3D pipeline.
    pub origin: Vec3,

    /// The [`ShapeData`] of this shape.
    pub data: T,
}

/// Trait implemented by each shapes shader data, defines common methods used in the rendering pipeline.
pub trait ShapeData: Send + Sync + GpuArrayBufferable + 'static {
    /// Corresponding component representing the given shape.
    type Component: ShapeComponent<Data = Self>;

    const VERTICES: u32 = 6;
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

    fn shader_defs(app: &App) -> Vec<ShaderDefVal> {
        let mut shader_defs = Vec::with_capacity(1);

        if let Ok(render_app) = app.get_sub_app(RenderApp) {
            if let Some(per_object_buffer_batch_size) =
                GpuArrayBuffer::<Self>::batch_size(render_app.world.resource::<RenderDevice>())
            {
                shader_defs.push(ShaderDefVal::UInt(
                    "PER_OBJECT_BUFFER_BATCH_SIZE".into(),
                    per_object_buffer_batch_size,
                ));
            }
        }

        shader_defs
    }
}

/// Trait implemented by the corresponding component for each shape type.
pub trait ShapeComponent: Component + GetTypeRegistration {
    type Data: ShapeData<Component = Self>;
    fn get_data(&self, tf: &GlobalTransform, fill: &ShapeFill) -> Self::Data;
}

/// Determines whether the shape is rendered in the 2D or 3D pipelines.
#[derive(Resource, Copy, Clone, Reflect, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub enum ShapePipelineType {
    Shape3d,
    Shape2d,
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
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Component)]
pub struct ShapePipelineMaterial {
    render_layers: RenderLayersHash,
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
            render_layers: RenderLayersHash(render_layers.cloned().unwrap_or_default()),
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
            render_layers: RenderLayersHash(config.render_layers.unwrap_or_default()),
            alpha_mode: AlphaModeOrd(config.alpha_mode),
            disable_laa: config.disable_laa || config.alpha_mode == AlphaMode::Opaque,
            texture: config.texture.clone(),
            pipeline: config.pipeline,
            canvas: config.canvas,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Ord, PartialOrd)]
struct RenderLayersHash(RenderLayers);

impl Hash for RenderLayersHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { std::mem::transmute::<_, &u32>(self).hash(state) }
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

impl Hash for AlphaModeOrd {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.ord()).hash(state);
    }
}

impl PartialOrd for AlphaModeOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AlphaModeOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.ord()).cmp(&FloatOrd(other.ord()))
    }
}

#[derive(Resource)]
pub struct QuadVertices {
    buffer: Buffer,
}

const QUAD: [[f32; 3]; 6] = [
    [-1.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [-1.0, 1.0, 0.0],
];

impl FromWorld for QuadVertices {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::VERTEX,
            label: Some("quad_vertex_buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    QUAD.as_ptr().cast(),
                    std::mem::size_of_val(&QUAD) / std::mem::size_of::<u8>(),
                )
            },
        });

        Self { buffer }
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
        .init_resource::<QuadVertices>()
        .add_systems(ExtractSchedule, extract_render_layers)
        .add_systems(
            Render,
            prepare_shape_view_bind_groups.in_set(RenderSet::PrepareBindGroups),
        )
        .add_systems(
            Render,
            prepare_shape_texture_bind_groups.in_set(RenderSet::PrepareBindGroups),
        );
}

fn setup_type_pipeline<T: ShapeData + 'static>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<ShapePipeline<T>>()
        .add_systems(
            Render,
            (
                write_batched_instance_buffer::<T>.in_set(RenderSet::PrepareResourcesFlush),
                prepare_shape_bind_group::<T>.in_set(RenderSet::PrepareBindGroups),
            ),
        );
}

fn setup_type_pipeline_3d<T: ShapeData + 'static>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, DrawShapeCommand<T>>()
        .add_render_command::<Transparent3d, DrawShapeCommand<T>>()
        .add_render_command::<AlphaMask3d, DrawShapeCommand<T>>()
        .init_resource::<Shape3dInstances<T>>()
        .init_resource::<Shape3dMaterials<T>>()
        .add_systems(ExtractSchedule, extract_shapes_3d::<T>)
        .add_systems(
            Render,
            (
                queue_shapes_3d::<T>.in_set(RenderSet::Queue),
                batch_and_prepare_render_phase::<T, Shape3dInstances<T>, Opaque3d>
                    .in_set(RenderSet::PrepareResources),
                batch_and_prepare_render_phase::<T, Shape3dInstances<T>, AlphaMask3d>
                    .in_set(RenderSet::PrepareResources),
                batch_and_prepare_render_phase::<T, Shape3dInstances<T>, Transparent3d>
                    .in_set(RenderSet::PrepareResources),
            ),
        );
}

fn setup_type_pipeline_2d<T: ShapeData + 'static>(app: &mut App) {
    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .insert_resource(GpuArrayBuffer::<T>::new(
                render_app.world.resource::<RenderDevice>(),
            ))
            .add_render_command::<Transparent2d, DrawShapeCommand<T>>()
            .init_resource::<Shape2dInstances<T>>()
            .init_resource::<Shape2dMaterials<T>>()
            .add_systems(ExtractSchedule, extract_shapes_2d::<T>)
            .add_systems(
                Render,
                (
                    queue_shapes_2d::<T>.in_set(RenderSet::Queue),
                    batch_and_prepare_render_phase::<T, Shape2dInstances<T>, Transparent2d>
                        .in_set(RenderSet::PrepareResources),
                ),
            );
    }
}

pub fn write_batched_instance_buffer<T: ShapeData + 'static>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_array_buffer: ResMut<GpuArrayBuffer<T>>,
) {
    let gpu_array_buffer = gpu_array_buffer.into_inner();
    gpu_array_buffer.write_buffer(&render_device, &render_queue);
    gpu_array_buffer.clear();
}

/// Plugin that sets up the 2d render pipeline for the given [`ShapeComponent`].
#[derive(Default)]
pub struct ShapeTypePlugin<T: ShapeComponent>(PhantomData<T>);

impl<T: ShapeComponent> Plugin for ShapeTypePlugin<T> {
    fn build(&self, app: &mut App) {
        app.register_type::<T>();
    }

    fn finish(&self, app: &mut App) {
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
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        load_shaders(app);
        setup_pipeline(app);
    }
}

// TODO: PR to bevy to make this public
#[derive(PartialEq)]
struct BatchMeta<T: PartialEq> {
    /// The pipeline id encompasses all pipeline configuration including vertex
    /// buffers and layouts, shaders and their specializations, bind group
    /// layouts, etc.
    pipeline_id: CachedRenderPipelineId,
    /// The draw function id defines the RenderCommands that are called to
    /// set the pipeline and bindings, and make the draw command
    draw_function_id: DrawFunctionId,
    dynamic_offset: Option<NonMaxU32>,
    pub user_data: T,
}

impl<T: PartialEq> BatchMeta<T> {
    fn new(item: &impl CachedRenderPipelinePhaseItem, user_data: T) -> Self {
        BatchMeta {
            pipeline_id: item.cached_pipeline(),
            draw_function_id: item.draw_function(),
            dynamic_offset: item.dynamic_offset(),
            user_data,
        }
    }
}

pub fn batch_and_prepare_render_phase<
    T: ShapeData,
    R: Resource + Deref<Target = EntityHashMap<ShapeInstance<T>>>,
    P: CachedRenderPipelinePhaseItem,
>(
    mut commands: Commands,
    mut gpu_array_buffer: ResMut<GpuArrayBuffer<T>>,
    mut views: Query<&mut RenderPhase<P>>,
    instance_data: Res<R>,
) {
    let mut process_item = |item: &mut P| {
        let instance = instance_data.get(&item.entity())?;
        let buffer_index = gpu_array_buffer.push(instance.data.clone());

        let index = buffer_index.index.get();
        *item.batch_range_mut() = index..index + 1;
        *item.dynamic_offset_mut() = buffer_index.dynamic_offset;

        Some(BatchMeta::new(item, &instance.material))
    };

    let mut batches = Vec::new();
    for mut phase in &mut views {
        let items = phase.items.iter_mut().map(|item| {
            let batch_data = process_item(item);
            (item.entity(), item.batch_range_mut(), batch_data)
        });
        let last = items.reduce(
            |(prev_entity, start_range, prev_batch_meta), (entity, range, batch_meta)| {
                if batch_meta.is_some() && prev_batch_meta == batch_meta {
                    start_range.end = range.end;
                    (prev_entity, start_range, prev_batch_meta)
                } else {
                    if let Some(prev_batch_meta) = prev_batch_meta {
                        batches.push((prev_entity, prev_batch_meta.user_data.clone()));
                    }
                    (entity, range, batch_meta)
                }
            },
        );
        if let Some((entity, _, Some(batch_meta))) = last {
            batches.push((entity, batch_meta.user_data.clone()));
        }
    }

    commands.insert_or_spawn_batch(batches);
}
