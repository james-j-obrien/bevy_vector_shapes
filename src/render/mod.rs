use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;

use bevy::asset::uuid_handle;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::entity::{EntityGeneration, EntityIndex};
use bevy::math::FloatOrd;
use bevy::render::batching::no_gpu_preprocessing::BatchedInstanceBuffer;
use bevy::render::render_phase::{
    DrawFunctions, PhaseItemExtraIndex, SortedPhaseItem, ViewSortedRenderPhases,
};
use bevy::render::sync_world::MainEntity;
use bevy::shader::ShaderDefVal;
use bevy::shader::ShaderRef;
use bevy::{
    asset::load_internal_asset,
    core_pipeline::{
        core_2d::Transparent2d,
        core_3d::{AlphaMask3d, Opaque3d, Transparent3d},
    },
    prelude::*,
    reflect::GetTypeRegistration,
    render::{
        render_phase::{AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId},
        render_resource::{Buffer, CachedRenderPipelineId, GpuArrayBuffer, GpuArrayBufferable},
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp, RenderSystems,
    },
};
use bitfield::bitfield;
use nonmax::NonMaxU32;
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
pub const CORE_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-b766-25c7b7116e7a");

/// Handler to shader containing shared constants.
pub const CONSTANTS_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-c98e-c4f33ff6f60b");

/// Handler to shader for drawing discs.
pub const DISC_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-ae5a-7141de1d0b16");

/// Handler to shader for drawing lines.
pub const LINE_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-bd87-2dd097d75b68");

/// Handler to shader for drawing regular polygons.
pub const NGON_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-f167-5038026cdfbb");

/// Handler to shader for drawing rectangles.
pub const RECT_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-d121-147b5fcad83f");

/// Handler to shader for drawing triangles.
pub const TRIANGLE_HANDLE: Handle<Shader> = uuid_handle!("00000000-0000-0000-ab4e-d06c34e4155f");

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
    let defs = LineData::shader_defs(app);
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
    let defs = TriangleData::shader_defs(app);
    load_internal_asset!(
        app,
        TRIANGLE_HANDLE,
        "shaders/shapes/tri.wgsl",
        Shader::from_wgsl_with_defs,
        defs
    );
}

/// Contains data necessary to render a single shape.
#[derive(Clone)]
pub struct ShapeInstance<T> {
    /// This shape's material.
    pub material: ShapePipelineMaterial,

    /// The point in space used for ordering this shape.
    /// Ignored by the 2D pipeline.
    pub origin: Vec3,

    /// The [`ShapeData`] of this shape.
    pub data: T,
}

/// Per-frame storage of extracted shape instances for one shape type, grouped by material.
pub struct ShapeInstanceBuffer<T: ShapeData> {
    /// Unique materials seen this frame
    pub materials: Vec<ShapePipelineMaterial>,
    /// (material index, shape data)
    pub instances: Vec<(u32, T)>,
    /// Ordering origin: `z` is the 2D sort key, the full point feeds the 3D rangefinder.
    pub origins: Vec<Vec3>,
    /// Instance indices grouped by material index.
    pub groups: Vec<Vec<u32>>,
}

impl<T: ShapeData> Default for ShapeInstanceBuffer<T> {
    fn default() -> Self {
        Self {
            materials: Vec::new(),
            instances: Vec::new(),
            origins: Vec::new(),
            groups: Vec::new(),
        }
    }
}

impl<T: ShapeData> ShapeInstanceBuffer<T> {
    pub fn clear(&mut self) {
        self.materials.clear();
        self.instances.clear();
        self.origins.clear();
        for group in &mut self.groups {
            group.clear();
        }
    }

    pub fn intern(&mut self, material: &ShapePipelineMaterial) -> u32 {
        if let Some(index) = self.materials.iter().position(|m| m == material) {
            index as u32
        } else {
            self.materials.push(material.clone());
            if self.groups.len() < self.materials.len() {
                self.groups.push(Vec::new());
            }
            (self.materials.len() - 1) as u32
        }
    }

    pub fn push(&mut self, material: u32, origin: Vec3, data: T) {
        let index = self.instances.len() as u32;
        self.instances.push((material, data));
        self.origins.push(origin);
        self.groups[material as usize].push(index);
    }
}

/// Stable per-shape-type id used to build fabricated phase-item keys.
#[derive(Resource)]
pub struct ShapeTypeTag<T> {
    tag: u32,
    _marker: PhantomData<T>,
}

/// Generation values for fabricated keys start here, hopefully no-one renders entities this old.
const FABRICATED_GENERATION_BASE: u32 = 0x4000_0000;

impl<T> ShapeTypeTag<T> {
    /// Builds the phase-item key for the instance at `index
    #[inline]
    pub fn phase_key(&self, index: u32) -> (Entity, MainEntity) {
        let entity = Entity::from_index_and_generation(
            EntityIndex::from_raw_u32(index).expect("shape instance index overflow"),
            EntityGeneration::from_bits(FABRICATED_GENERATION_BASE | self.tag),
        );
        (entity, MainEntity::from(entity))
    }
}

#[derive(Resource, Default)]
struct ShapeTypeTagAllocator(u32);

/// Connects a [`ShapeInstanceBuffer`] resource to the phase item type it queues into,
/// so [`batch_and_prepare_render_phase`] can look up instances and identify its items.
pub trait ShapeBatchSource<I: CachedRenderPipelinePhaseItem + SortedPhaseItem>:
    Resource + std::ops::Deref<Target = ShapeInstanceBuffer<Self::Data>>
{
    type Data: ShapeData;
    /// The draw command registered for this phase, used to identify our items.
    type Command: 'static;
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

        if let Some(render_app) = app.get_sub_app(RenderApp) {
            if let Some(per_object_buffer_batch_size) = GpuArrayBuffer::<Self>::batch_size(
                &render_app.world().resource::<RenderDevice>().limits(),
            ) {
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
    alpha_mode: ShapeAlphaMode,
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
            alpha_mode: material.alpha_mode,
            disable_laa: material.disable_laa,
            //|| material.alpha_mode == AlphaMode::Opaque
            canvas: material.canvas,
            pipeline: material.pipeline,
            texture: material.texture,
        }
    }
}

impl From<&ShapeConfig> for ShapePipelineMaterial {
    fn from(config: &ShapeConfig) -> Self {
        Self {
            render_layers: RenderLayersHash(config.render_layers.clone().unwrap_or_default()),
            alpha_mode: config.alpha_mode,
            disable_laa: config.disable_laa,
            // || material.alpha_mode == AlphaMode::Opaque
            texture: config.texture.clone(),
            pipeline: config.pipeline,
            canvas: config.canvas,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Ord, PartialOrd)]
struct RenderLayersHash(RenderLayers);

impl Hash for RenderLayersHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.bits().hash(state)
    }
}

#[derive(Resource)]
pub struct QuadVertices {
    pub(crate) buffer: Buffer,
    pub(crate) indices: Buffer,
}

const QUAD: [[f32; 3]; 4] = [
    [-1.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

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
        let indices = render_device.create_buffer_with_data(&BufferInitDescriptor {
            usage: BufferUsages::INDEX,
            label: Some("quad_index_buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    QUAD_INDICES.as_ptr().cast(),
                    std::mem::size_of_val(&QUAD_INDICES) / std::mem::size_of::<u8>(),
                )
            },
        });

        Self { buffer, indices }
    }
}

fn setup_pipeline(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .init_resource::<ShapePipelines>()
        .init_resource::<ShapeTextureBindGroups>()
        .init_resource::<QuadVertices>()
        .add_systems(
            Render,
            prepare_shape_view_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
}

fn setup_type_pipeline<T: ShapeData + 'static>(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    let tag = {
        let mut allocator = render_app
            .world_mut()
            .get_resource_or_insert_with(ShapeTypeTagAllocator::default);
        let tag = allocator.0;
        allocator.0 += 1;
        tag
    };
    render_app
        .insert_resource(ShapeTypeTag::<T> {
            tag,
            _marker: PhantomData,
        })
        .add_systems(
            Render,
            write_batched_instance_buffer::<T>.in_set(RenderSystems::PrepareResourcesFlush),
        );
}

fn setup_type_pipeline_3d<T: ShapeData + 'static>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_render_command::<Opaque3d, DrawShape3dCommand<T>>()
        .add_render_command::<Transparent3d, DrawShape3dCommand<T>>()
        .add_render_command::<AlphaMask3d, DrawShape3dCommand<T>>()
        .init_resource::<Shape3dInstanceBuffer<T>>()
        .init_resource::<Shape3dPipeline<T>>()
        .add_systems(ExtractSchedule, extract_shapes_3d::<T>)
        .add_systems(
            Render,
            (
                prepare_shape_3d_bind_group::<T>.in_set(RenderSystems::PrepareBindGroups),
                prepare_shape_3d_texture_bind_groups::<T>.in_set(RenderSystems::PrepareBindGroups),
                queue_shapes_3d::<T>.in_set(RenderSystems::Queue),
                batch_and_prepare_render_phase::<Transparent3d, Shape3dInstanceBuffer<T>>
                    .in_set(RenderSystems::PrepareResources),
            ),
        );
}

fn setup_type_pipeline_2d<T: ShapeData + 'static>(app: &mut App) {
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .insert_resource(BatchedInstanceBuffer::<T>::new(
                &render_app.world().resource::<RenderDevice>().limits(),
            ))
            .add_render_command::<Transparent2d, DrawShape2dCommand<T>>()
            .init_resource::<Shape2dInstanceBuffer<T>>()
            .init_resource::<Shape2dPipeline<T>>()
            .add_systems(ExtractSchedule, extract_shapes_2d::<T>)
            .add_systems(
                Render,
                (
                    prepare_shape_2d_bind_group::<T>.in_set(RenderSystems::PrepareBindGroups),
                    prepare_shape_2d_texture_bind_groups::<T>
                        .in_set(RenderSystems::PrepareBindGroups),
                    queue_shapes_2d::<T>.in_set(RenderSystems::Queue),
                    batch_and_prepare_render_phase::<Transparent2d, Shape2dInstanceBuffer<T>>
                        .in_set(RenderSystems::PrepareResources),
                ),
            );
    }
}

pub fn write_batched_instance_buffer<T: ShapeData + 'static>(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_array_buffer: ResMut<BatchedInstanceBuffer<T>>,
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
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
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
            dynamic_offset: match item.extra_index() {
                PhaseItemExtraIndex::DynamicOffset(dynamic_offset) => {
                    NonMaxU32::new(dynamic_offset)
                }
                _ => None,
            },
            user_data,
        }
    }
}

pub fn batch_and_prepare_render_phase<I, S>(
    mut gpu_array_buffer: ResMut<BatchedInstanceBuffer<S::Data>>,
    mut phases: ResMut<ViewSortedRenderPhases<I>>,
    source: Res<S>,
    draw_functions: Res<DrawFunctions<I>>,
) where
    I: CachedRenderPipelinePhaseItem + SortedPhaseItem,
    S: ShapeBatchSource<I>,
{
    let draw_function = draw_functions.read().id::<S::Command>();

    let mut process_item = |item: &mut I| {
        // Only process items we own
        if item.draw_function() != draw_function {
            return None;
        }
        let instance_index = item.main_entity().id().index_u32();
        let (material, data) = source.instances.get(instance_index as usize)?;
        let buffer_index = gpu_array_buffer.push(data.clone());

        let index = buffer_index.index;
        let (item_batch, item_index) = item.batch_range_and_extra_index_mut();
        *item_batch = index..index + 1;
        *item_index = PhaseItemExtraIndex::maybe_dynamic_offset(buffer_index.dynamic_offset);

        Some(*material)
    };

    for phase in phases.values_mut() {
        let items = phase.items.values_mut().map(|item| {
            let batch_data = process_item(item).map(|c| BatchMeta::new(item, c));

            (item.batch_range_mut(), batch_data)
        });

        items.reduce(|(start_range, prev_batch_meta), (range, batch_meta)| {
            if batch_meta.is_some() && prev_batch_meta == batch_meta {
                start_range.end = range.end;
                (start_range, prev_batch_meta)
            } else {
                (range, batch_meta)
            }
        });
    }
}
