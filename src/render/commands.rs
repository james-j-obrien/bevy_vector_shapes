use std::marker::PhantomData;

use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::Read, SystemParamItem},
    },
    prelude::*,
    render::{
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
};

use crate::render::*;

pub type DrawInstancedCommand<T> = (
    SetItemPipeline,
    SetInstancedViewBindGroup<T, 0>,
    DrawInstanced<T>,
);

#[derive(Component, Debug)]
pub struct InstancedViewBindGroup<T: ShapeData> {
    value: BindGroup,
    _marker: PhantomData<T>,
}

pub fn queue_instance_view_bind_groups<T: ShapeData>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    instanced_pipeline: Res<InstancedPipeline<T>>,
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
                label: Some("instanced_view_bind_group"),
                layout: &instanced_pipeline.view_layout,
            });

            commands.entity(entity).insert(InstancedViewBindGroup::<T> {
                value: view_bind_group,
                _marker: default(),
            });
        }
    }
}

pub struct SetInstancedViewBindGroup<T: ShapeData, const I: usize>(PhantomData<T>);

impl<T: ShapeData, const I: usize, P: PhaseItem> RenderCommand<P>
    for SetInstancedViewBindGroup<T, I>
{
    type ViewWorldQuery = (Read<ViewUniformOffset>, Read<InstancedViewBindGroup<T>>);
    type ItemWorldQuery = ();
    type Param = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        (view_uniform, instanced_view_bind_group): ROQueryItem<'w, Self::ViewWorldQuery>,
        _entity: ROQueryItem<'w, Self::ItemWorldQuery>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &instanced_view_bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

pub struct DrawInstanced<T> {
    _marker: PhantomData<T>,
}

impl<P: PhaseItem, T: ShapeData + 'static> RenderCommand<P> for DrawInstanced<T> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<InstanceBuffer<T>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        instance_buffer: &'w InstanceBuffer<T>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_vertex_buffer(0, instance_buffer.buffer.slice(..));
        pass.draw(0..6, 0..instance_buffer.length as u32);

        RenderCommandResult::Success
    }
}
