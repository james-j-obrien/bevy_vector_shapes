use std::any::TypeId;

use bevy::{
    core_pipeline::tonemapping::get_lut_bind_group_layout_entries,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    render::{
        globals::GlobalsUniform, render_resource::*, renderer::RenderDevice,
        sync_world::MainEntity, view::ViewUniform,
    },
    utils::HashMap,
};
use binding_types::uniform_buffer;
use wgpu::vertex_attr_array;

use super::*;

bitflags::bitflags! {
    #[derive(Eq, PartialEq, Hash, Clone, Copy)]
    #[repr(transparent)]
    pub struct ShapePipelineKey: u32 {
        const NONE                              = 0;
        const HDR                               = (1 << 0);
        const PIPELINE_2D                       = (1 << 2);
        const LOCAL_AA                          = (1 << 3);
        const TEXTURED                          = (1 << 4);
        const BLEND_RESERVED_BITS               = Self::BLEND_MASK_BITS << Self::BLEND_SHIFT_BITS;
        const BLEND_OPAQUE                      = (0 << Self::BLEND_SHIFT_BITS);
        const BLEND_ADD                         = (1 << Self::BLEND_SHIFT_BITS);
        const BLEND_MULTIPLY                    = (2 << Self::BLEND_SHIFT_BITS);
        const BLEND_ALPHA                       = (3 << Self::BLEND_SHIFT_BITS);
        const MSAA_RESERVED_BITS                = Self::MSAA_MASK_BITS << Self::MSAA_SHIFT_BITS;
    }
}

impl ShapePipelineKey {
    const MSAA_MASK_BITS: u32 = 0b111;
    const MSAA_SHIFT_BITS: u32 = 32 - Self::MSAA_MASK_BITS.count_ones();
    const BLEND_MASK_BITS: u32 = 0b11;
    const BLEND_SHIFT_BITS: u32 = Self::MSAA_SHIFT_BITS - Self::BLEND_MASK_BITS.count_ones();

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits =
            (msaa_samples.trailing_zeros() & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        Self::from_bits_retain(msaa_bits)
    }

    pub fn from_hdr(hdr: bool) -> Self {
        if hdr {
            ShapePipelineKey::HDR
        } else {
            ShapePipelineKey::NONE
        }
    }

    pub fn msaa_samples(&self) -> u32 {
        1 << ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS)
    }

    pub fn from_material(material: &ShapePipelineMaterial) -> Self {
        let mut key = match material.alpha_mode {
            ShapeAlphaMode::Add => Self::BLEND_ADD,
            ShapeAlphaMode::Multiply => Self::BLEND_MULTIPLY,
            _ => Self::BLEND_ALPHA,
        };
        if material.texture.is_some() {
            key |= Self::TEXTURED;
        }

        key
    }
}

#[derive(Resource)]
pub struct ShapePipelines {
    pub view_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pipeline_cache: HashMap<(ShapePipelineKey, TypeId), CachedRenderPipelineId>,
}

impl FromWorld for ShapePipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let tonemapping_lut_entries = get_lut_bind_group_layout_entries();
        let view_layout = render_device.create_bind_group_layout(
            "shape_view_layout",
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    (0, uniform_buffer::<ViewUniform>(true)),
                    (1, uniform_buffer::<GlobalsUniform>(false)),
                    (
                        2,
                        tonemapping_lut_entries[0].visibility(ShaderStages::FRAGMENT),
                    ),
                    (
                        3,
                        tonemapping_lut_entries[1].visibility(ShaderStages::FRAGMENT),
                    ),
                ),
            ),
        );
        let texture_layout = render_device.create_bind_group_layout(
            Some("shape_texture_layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        );

        Self {
            view_layout,
            texture_layout,
            pipeline_cache: default(),
        }
    }
}

impl ShapePipelines {
    pub fn specialize<T: ShapeData + 'static>(
        &mut self,
        cache: &PipelineCache,
        pipeline: &Shape2dPipeline<T>,
        key: ShapePipelineKey,
    ) -> CachedRenderPipelineId {
        let Self {
            view_layout,
            texture_layout,
            pipeline_cache,
        } = self;

        *pipeline_cache
            .entry((key, TypeId::of::<T>()))
            .or_insert_with(|| {
                let descriptor =
                    pipeline.specialize(view_layout, texture_layout, &pipeline.layout, key);
                cache.queue_render_pipeline(descriptor)
            })
    }
}

#[derive(Resource)]
pub struct Shape2dPipeline<T: ShapeData> {
    pub shader: Handle<Shader>,
    pub layout: BindGroupLayout,
    _marker: PhantomData<T>,
}

impl<T: ShapeData> FromWorld for Shape2dPipeline<T> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            Some("shape_layout"),
            &BindGroupLayoutEntries::with_indices(
                ShaderStages::VERTEX,
                ((0, GpuArrayBuffer::<T>::binding_layout(render_device)),),
            ),
        );

        let asset_server = world.resource_mut::<AssetServer>();
        Self {
            layout,
            shader: match T::shader() {
                ShaderRef::Default => RECT_HANDLE,
                ShaderRef::Handle(handle) => handle,
                ShaderRef::Path(path) => asset_server.load(path),
            },
            _marker: default(),
        }
    }
}

impl<T: ShapeData> Shape2dPipeline<T> {
    fn specialize(
        &self,
        view_layout: &BindGroupLayout,
        texture_layout: &BindGroupLayout,
        shape_layout: &BindGroupLayout,
        key: ShapePipelineKey,
    ) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        let (label, blend, depth_stencil, depth_write_enabled);

        let pass = key.intersection(ShapePipelineKey::BLEND_RESERVED_BITS);

        if pass == ShapePipelineKey::BLEND_ALPHA {
            label = "alpha_blend_shape_pipeline".into();
            blend = Some(BlendState::ALPHA_BLENDING);
            shader_defs.push("BLEND_ALPHA".into());
            depth_write_enabled = false;
        } else if pass == ShapePipelineKey::BLEND_ADD {
            label = "add_blend_shape_pipeline".into();
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
        } else if pass == ShapePipelineKey::BLEND_MULTIPLY {
            label = "multiply_blend_shape_pipeline".into();
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
            label = "opaque_shape_pipeline".into();
            blend = Some(BlendState::REPLACE);
            shader_defs.push("BLEND_ALPHA".into());
            depth_write_enabled = true;
        }

        if key.contains(ShapePipelineKey::PIPELINE_2D) {
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

        if key.contains(ShapePipelineKey::LOCAL_AA) {
            shader_defs.push("LOCAL_AA".into());
        } else {
            shader_defs.push("DISABLE_LOCAL_AA".into())
        }

        let format = match key.contains(ShapePipelineKey::HDR) {
            true => bevy::render::view::ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        let mut layout = vec![view_layout.clone(), shape_layout.clone()];
        if key.contains(ShapePipelineKey::TEXTURED) {
            layout.push(texture_layout.clone());
            shader_defs.push("TEXTURED".into());
        }

        let mut fragment_defs = shader_defs.clone();
        fragment_defs.push("FRAGMENT".into());

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vertex_attr_array![0 => Float32x3].into(),
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
            layout,
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
            zero_initialize_workgroup_memory: false,
        }
    }
}

impl<T: ShapeData> GetBatchData for Shape2dPipeline<T> {
    type Param = SRes<Shape2dInstances<T>>;
    type CompareData = ShapePipelineMaterial;
    type BufferData = T;

    fn get_batch_data(
        instances: &SystemParamItem<Self::Param>,
        (entity, _main_entity): (Entity, MainEntity),
    ) -> Option<(Self::BufferData, Option<Self::CompareData>)> {
        let instance = instances.get(&entity)?;
        Some((instance.data.clone(), Some(instance.material.clone())))
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Shape3dPipeline<T: ShapeData>(Shape2dPipeline<T>);

impl<T: ShapeData> FromWorld for Shape3dPipeline<T> {
    fn from_world(world: &mut World) -> Self {
        Self(Shape2dPipeline::<T>::from_world(world))
    }
}

impl<T: ShapeData> GetBatchData for Shape3dPipeline<T> {
    type Param = SRes<Shape3dInstances<T>>;
    type CompareData = ShapePipelineMaterial;
    type BufferData = T;

    fn get_batch_data(
        instances: &SystemParamItem<Self::Param>,
        (entity, _main_entity): (Entity, MainEntity),
    ) -> Option<(Self::BufferData, Option<Self::CompareData>)> {
        let instance = instances.get(&entity)?;
        Some((instance.data.clone(), Some(instance.material.clone())))
    }
}
