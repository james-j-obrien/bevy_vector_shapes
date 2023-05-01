use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        render_phase::{
            BatchedPhaseItem, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupLayout, BufferVec, FragmentState, PipelineCache,
            RenderPipelineDescriptor, ShaderDefVal, ShaderType, SpecializedRenderPipeline,
            SpecializedRenderPipelines, VertexBufferLayout, VertexState,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, RenderLayers, ViewUniform, ViewUniformOffset, ViewUniforms},
        Extract,
    },
    utils::FloatOrd,
};
use bytemuck::{Pod, Zeroable};
use wgpu::{
    vertex_attr_array, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState,
    BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
    PrimitiveTopology, ShaderStages, StencilFaceState, StencilState, TextureFormat, VertexStepMode,
};

use crate::prelude::{Shape, ShapeEvent};

use super::{
    pipeline::InstancedPipelineKey,
    render_resource::{GpuList, GpuListIndex},
    InstanceComponent, Instanceable, RenderKey,
};

#[derive(Resource)]
pub struct ShapePipeline<T: Instanceable> {
    view_layout: BindGroupLayout,
    shape_layout: BindGroupLayout,
    shape_buffer_batch_size: Option<u32>,
    shader: Handle<Shader>,
    _marker: PhantomData<T>,
}
impl<T: Instanceable> FromWorld for ShapePipeline<T> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ViewUniform::min_size()),
                },
                count: None,
            }],
            label: Some("sprite_view_layout"),
        });

        let shape_binding =
            GpuList::<T>::binding_layout(0, ShaderStages::VERTEX_FRAGMENT, render_device);

        let shape_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[shape_binding],
            label: Some("shape_layout"),
        });

        Self {
            view_layout,
            shape_layout,
            shape_buffer_batch_size: GpuList::<T>::batch_size(&render_device),
            shader: T::shader(),
            _marker: default(),
        }
    }
}

impl<T: Instanceable> SpecializedRenderPipeline for ShapePipeline<T> {
    type Key = InstancedPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        let (label, blend, depth_stencil, depth_write_enabled);

        if let Some(batch_size) = self.shape_buffer_batch_size {
            shader_defs.push(ShaderDefVal::UInt("BATCH_SIZE".into(), batch_size));
        }

        let pass = key.intersection(InstancedPipelineKey::BLEND_RESERVED_BITS);

        if pass == InstancedPipelineKey::BLEND_ALPHA {
            label = "alpha_blend_batched_pipeline".into();
            blend = Some(BlendState::ALPHA_BLENDING);
            shader_defs.push("BLEND_ALPHA".into());
            depth_write_enabled = false;
        } else if pass == InstancedPipelineKey::BLEND_ADD {
            label = "add_blend_batched_pipeline".into();
            blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
            });
            shader_defs.push("BLEND_ADD".into());
            depth_write_enabled = false;
        } else if pass == InstancedPipelineKey::BLEND_MULTIPLY {
            label = "multiply_blend_batched_pipeline".into();
            blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::Dst,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
            shader_defs.push("BLEND_MULTIPLY".into());
            depth_write_enabled = false;
        } else {
            label = "opaque_batched_pipeline".into();
            blend = Some(BlendState::REPLACE);
            shader_defs.push("BLEND_ALPHA".into());
            depth_write_enabled = true;
        }

        if key.contains(InstancedPipelineKey::PIPELINE_2D) {
            depth_stencil = None;
            shader_defs.push("PIPELINE_2D".into());
        } else {
            depth_stencil = Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            });
            shader_defs.push("PIPELINE_3D".into());
        }

        if key.contains(InstancedPipelineKey::LOCAL_AA) {
            shader_defs.push("LOCAL_AA".into());
        } else {
            shader_defs.push("DISABLE_LOCAL_AA".into())
        }

        let format = match key.contains(InstancedPipelineKey::HDR) {
            true => bevy::render::view::ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        let mut fragment_defs = shader_defs.clone();
        fragment_defs.push("FRAGMENT".into());

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![VertexBufferLayout {
                    array_stride: std::mem::size_of::<u32>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attr_array![
                        0 => Uint32
                    ]
                    .to_vec(),
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: fragment_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![self.view_layout.clone(), self.shape_layout.clone()],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some(label),
            push_constant_ranges: vec![],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ShapeVertex {
    shape_index: u32,
}

#[derive(Resource)]
pub struct ShapeMeta<T: Instanceable> {
    view_bind_group: Option<BindGroup>,
    shape_bind_group: Option<BindGroup>,
    vertices: BufferVec<ShapeVertex>,
    _marker: PhantomData<T>,
}

impl<T: Instanceable> Default for ShapeMeta<T> {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
            shape_bind_group: None,
            _marker: default(),
        }
    }
}

#[derive(Resource)]
pub struct ExtractedShapes<T: Instanceable> {
    shapes: Vec<(RenderKey, T)>,
}

impl<T: Instanceable> Default for ExtractedShapes<T> {
    fn default() -> Self {
        Self {
            shapes: Default::default(),
        }
    }
}

pub fn extract_shapes<T: Instanceable>(
    mut extracted_shapes: ResMut<ExtractedShapes<T>>,
    entities: Extract<
        Query<(
            &T::Component,
            &GlobalTransform,
            &ComputedVisibility,
            Option<&Shape>,
            Option<&RenderLayers>,
        )>,
    >,
    mut events: Extract<EventReader<ShapeEvent<T>>>,
) {
    extracted_shapes.shapes.clear();
    extracted_shapes.shapes = entities
        .iter()
        .filter_map(|(cp, tf, vis, flags, rl)| {
            if vis.is_visible() {
                Some((RenderKey::new(flags, rl), cp.instance(tf)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    extracted_shapes
        .shapes
        .extend(events.into_iter().map(|e| e.0));
}

pub fn queue_shapes<T: Instanceable>(
    mut commands: Commands,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    view_uniforms: Res<ViewUniforms>,
    mut shape_meta: ResMut<ShapeMeta<T>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    shape_pipeline: Res<ShapePipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ShapePipeline<T>>>,
    msaa: Res<Msaa>,
    mut shape_uniforms: ResMut<GpuList<T>>,
    mut extracted_shapes: ResMut<ExtractedShapes<T>>,
    mut views: Query<(
        &ExtractedView,
        &mut RenderPhase<Transparent2d>,
        Option<&RenderLayers>,
    )>,
) {
    shape_meta.vertices.clear();
    shape_uniforms.clear();

    if extracted_shapes.shapes.is_empty() {
        return;
    }

    let draw_function = draw_functions.read().id::<DrawShape<T>>();

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        shape_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_binding,
            }],
            label: Some("shape_view_bind_group"),
            layout: &shape_pipeline.view_layout,
        }));

        let msaa_key = InstancedPipelineKey::from_msaa_samples(msaa.samples())
            | InstancedPipelineKey::PIPELINE_2D;
        let extracted_shapes = &mut extracted_shapes.shapes;
        extracted_shapes.sort_unstable_by_key(|(k, i)| (*k, FloatOrd(i.distance())));

        let mut vertex_index = 0;

        for (view, mut transparent_phase, render_layers) in &mut views {
            let view_key = InstancedPipelineKey::from_hdr(view.hdr) | msaa_key;
            let mut current_batch = ShapeBatch::<T>::PLACE_HOLDER;
            let mut current_batch_entity = Entity::PLACEHOLDER;

            let render_layers = render_layers.cloned().unwrap_or_default();

            for (key, shape) in extracted_shapes.iter() {
                if !render_layers.intersects(&key.render_layers) {
                    continue;
                }

                let new_batch = ShapeBatch::new(*key);
                let mut key =
                    InstancedPipelineKey::from_alpha_mode(new_batch.key.alpha_mode.0) | view_key;
                if !new_batch.key.disable_laa {
                    key |= InstancedPipelineKey::LOCAL_AA;
                }

                let item_start = vertex_index;
                vertex_index += 6;
                let item_end = vertex_index;

                let index = shape_uniforms.push(shape.clone());
                let vertex = ShapeVertex {
                    shape_index: index.index,
                };

                for _ in 0..6 {
                    shape_meta.vertices.push(vertex.clone());
                }

                if current_batch != new_batch {
                    current_batch = new_batch;
                    current_batch_entity = commands.spawn((current_batch.clone(), index)).id();
                }

                let pipeline = pipelines.specialize(&pipeline_cache, &shape_pipeline, key);
                transparent_phase.add(Transparent2d {
                    entity: current_batch_entity,
                    pipeline,
                    draw_function,
                    sort_key: FloatOrd(shape.distance()),
                    batch_range: Some(item_start..item_end),
                });
            }
        }
        shape_meta
            .vertices
            .write_buffer(&render_device, &render_queue);

        shape_uniforms.write_buffer(&render_device, &render_queue);

        shape_meta.shape_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: shape_uniforms.binding().unwrap(),
            }],
            label: Some("shape_bind_group"),
            layout: &shape_pipeline.shape_layout,
        }));
    }
}

pub type DrawShape<T> = (
    SetItemPipeline,
    SetShapeViewBindGroup<T, 0>,
    SetShapeBindGroup<T, 1>,
    DrawShapeBatch<T>,
);

pub struct SetShapeViewBindGroup<T: Instanceable, const I: usize>(PhantomData<T>);
impl<T: Instanceable, const I: usize, P: PhaseItem> RenderCommand<P>
    for SetShapeViewBindGroup<T, I>
{
    type Param = SRes<ShapeMeta<T>>;
    type ViewWorldQuery = Read<ViewUniformOffset>;
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        view_uniform: ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: ROQueryItem<'w, Self::ItemWorldQuery>,
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            shape_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}

pub struct SetShapeBindGroup<T: Instanceable, const I: usize>(PhantomData<T>);
impl<T: Instanceable, const I: usize, P: PhaseItem> RenderCommand<P> for SetShapeBindGroup<T, I> {
    type Param = SRes<ShapeMeta<T>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<GpuListIndex<T>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        index: &'_ GpuListIndex<T>,
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mut indices = Vec::new();
        if let Some(offset) = index.dynamic_offset {
            indices.push(offset);
        }

        pass.set_bind_group(
            I,
            &shape_meta.into_inner().shape_bind_group.as_ref().unwrap(),
            &indices,
        );
        RenderCommandResult::Success
    }
}

#[derive(Component, Copy, Clone)]
pub struct ShapeBatch<T: Instanceable> {
    key: RenderKey,
    _marker: PhantomData<T>,
}

impl<T: Instanceable> PartialEq for ShapeBatch<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T: Instanceable> ShapeBatch<T> {
    pub const PLACE_HOLDER: Self = Self::new(RenderKey::PLACE_HOLDER);

    pub const fn new(key: RenderKey) -> Self {
        Self {
            key,
            _marker: PhantomData::<T>,
        }
    }
}

pub struct DrawShapeBatch<T: Instanceable>(PhantomData<T>);
impl<P: BatchedPhaseItem, T: Instanceable> RenderCommand<P> for DrawShapeBatch<T> {
    type Param = SRes<ShapeMeta<T>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    fn render<'w>(
        item: &P,
        _view: (),
        _item: (),
        shape_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_vertex_buffer(
            0,
            shape_meta.into_inner().vertices.buffer().unwrap().slice(..),
        );
        pass.draw(item.batch_range().as_ref().unwrap().clone(), 0..1);
        RenderCommandResult::Success
    }
}
