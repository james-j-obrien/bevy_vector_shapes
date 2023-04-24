use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice, texture::BevyDefault, view::ViewUniform},
};

use super::*;

bitflags::bitflags! {
    #[derive(Eq, PartialEq, Hash, Clone, Copy)]
    #[repr(transparent)]
    pub struct InstancedPipelineKey: u32 {
        const NONE                              = 0;
        const HDR                               = (1 << 0);
        const PIPELINE_2D                       = (1 << 2);
        const LOCAL_AA                          = (1 << 3);
        const BLEND_RESERVED_BITS               = Self::BLEND_MASK_BITS << Self::BLEND_SHIFT_BITS;
        const BLEND_OPAQUE                      = (0 << Self::BLEND_SHIFT_BITS);
        const BLEND_ADD                         = (1 << Self::BLEND_SHIFT_BITS);
        const BLEND_MULTIPLY                    = (2 << Self::BLEND_SHIFT_BITS);
        const BLEND_ALPHA                       = (3 << Self::BLEND_SHIFT_BITS);
        const MSAA_RESERVED_BITS                = Self::MSAA_MASK_BITS << Self::MSAA_SHIFT_BITS;
    }
}

impl InstancedPipelineKey {
    const MSAA_MASK_BITS: u32 = 0b111;
    const MSAA_SHIFT_BITS: u32 = 32 - Self::MSAA_MASK_BITS.count_ones();
    const BLEND_MASK_BITS: u32 = 0b11;
    const BLEND_SHIFT_BITS: u32 = Self::MSAA_MASK_BITS - Self::BLEND_MASK_BITS.count_ones();

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits =
            (msaa_samples.trailing_zeros() & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        Self::from_bits_retain(msaa_bits)
    }

    pub fn from_hdr(hdr: bool) -> Self {
        if hdr {
            InstancedPipelineKey::HDR
        } else {
            InstancedPipelineKey::NONE
        }
    }

    pub fn msaa_samples(&self) -> u32 {
        1 << ((self.bits() >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS)
    }

    pub fn from_alpha_mode(alpha_mode: AlphaMode) -> Self {
        match alpha_mode {
            AlphaMode::Opaque => Self::BLEND_OPAQUE,
            AlphaMode::Mask(_) => Self::BLEND_OPAQUE,
            AlphaMode::Blend => Self::BLEND_ALPHA,
            AlphaMode::Premultiplied => Self::BLEND_ALPHA,
            AlphaMode::Add => Self::BLEND_ADD,
            AlphaMode::Multiply => Self::BLEND_MULTIPLY,
        }
    }
}

#[derive(Resource)]
pub struct InstancedPipeline<T: Instanceable> {
    pub view_layout: BindGroupLayout,
    shader: Handle<Shader>,
    _marker: PhantomData<T>,
}

impl<T: Instanceable> FromWorld for InstancedPipeline<T> {
    fn from_world(world: &mut World) -> Self {
        let shader = T::shader();

        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // View
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(ViewUniform::min_size()),
                    },
                    count: None,
                },
            ],
            label: Some("instanced_view_layout"),
        });

        Self {
            view_layout,
            shader,
            _marker: default(),
        }
    }
}

impl<T: Instanceable> SpecializedRenderPipeline for InstancedPipeline<T> {
    type Key = InstancedPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        let (label, blend, depth_stencil, depth_write_enabled);

        let pass = key.intersection(InstancedPipelineKey::BLEND_RESERVED_BITS);

        if pass == InstancedPipelineKey::BLEND_ALPHA {
            label = "alpha_blend_instanced_pipeline".into();
            blend = Some(BlendState::ALPHA_BLENDING);
            shader_defs.push("BLEND_ALPHA".into());
            depth_write_enabled = false;
        } else if pass == InstancedPipelineKey::BLEND_ADD {
            label = "add_blend_instanced_pipeline".into();
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
            label = "multiply_blend_instanced_pipeline".into();
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
            label = "opaque_instanced_pipeline".into();
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

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![VertexBufferLayout {
                    array_stride: std::mem::size_of::<T>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: T::vertex_layout(),
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![self.view_layout.clone()],
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
