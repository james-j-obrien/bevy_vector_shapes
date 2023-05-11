use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::BindGroup,
        view::ViewUniformOffset,
    },
    render::{
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, ViewUniforms},
    },
    utils::HashMap,
};

use crate::render::*;

pub type DrawShapeCommand = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetShapeTextureBindGroup<1>,
    DrawShape,
);

#[derive(Component, Debug)]
pub struct ShapeViewBindGroup {
    value: BindGroup,
}

pub fn queue_shape_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    shape_pipeline: Res<ShapePipelines>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter() {
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
                label: Some("shape_view_bind_group"),
                layout: &shape_pipeline.view_layout,
            });

            commands.entity(entity).insert(ShapeViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

pub struct SetShapeViewBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetShapeViewBindGroup<I> {
    type ViewWorldQuery = (Read<ViewUniformOffset>, Read<ShapeViewBindGroup>);
    type ItemWorldQuery = ();
    type Param = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        (view_uniform, shape_view_bind_group): ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &shape_view_bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

#[derive(Resource, Default)]
pub struct ShapeTextureBindGroups {
    values: HashMap<Handle<Image>, BindGroup>,
}

pub fn queue_shape_texture_bind_groups(
    render_device: Res<RenderDevice>,
    shape_pipelines: Res<ShapePipelines>,
    batches: Query<&ShapeDataBuffer>,
    gpu_images: Res<RenderAssets<Image>>,
    mut image_bind_groups: ResMut<ShapeTextureBindGroups>,
) {
    for buffer in batches.iter() {
        if let Some(handle) = &buffer.material.texture {
            if let Some(gpu_image) = gpu_images.get(&handle.cast_weak()) {
                image_bind_groups
                    .values
                    .entry(handle.cast_weak())
                    .or_insert_with(|| {
                        render_device.create_bind_group(&BindGroupDescriptor {
                            label: Some("shape_texture_bind_group"),
                            layout: &shape_pipelines.texture_layout,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::TextureView(&gpu_image.texture_view),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::Sampler(&gpu_image.sampler),
                                },
                            ],
                        })
                    });
            }
        }
    }
}

pub struct SetShapeTextureBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetShapeTextureBindGroup<I> {
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<ShapeDataBuffer>;
    type Param = SRes<ShapeTextureBindGroups>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        shape_buffer: &'w ShapeDataBuffer,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(handle) = &shape_buffer.material.texture {
            let bind_groups = bind_groups.into_inner();
            pass.set_bind_group(I, bind_groups.values.get(&handle.cast_weak()).unwrap(), &[]);
        }
        RenderCommandResult::Success
    }
}

pub struct DrawShape;

impl<P: PhaseItem> RenderCommand<P> for DrawShape {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<ShapeDataBuffer>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        shape_buffer: &'w ShapeDataBuffer,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_vertex_buffer(0, shape_buffer.buffer.slice(..));
        pass.draw(0..6, 0..shape_buffer.length as u32);

        RenderCommandResult::Success
    }
}
