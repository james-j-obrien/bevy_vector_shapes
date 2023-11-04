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

pub type DrawShapeCommand<T> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetShapeBindGroup<T, 1>,
    SetShapeTextureBindGroup<2>,
    DrawShape<T>,
);

#[derive(Component, Debug)]
pub struct ShapeViewBindGroup {
    value: BindGroup,
}

pub fn prepare_shape_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    shape_pipeline: Res<ShapePipelines>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter() {
            let view_bind_group = render_device.create_bind_group(
                "shape_view_bind_group",
                &shape_pipeline.view_layout,
                &BindGroupEntries::single(view_binding.clone()),
            );

            commands.entity(entity).insert(ShapeViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

#[derive(Resource, Default)]
pub struct ShapeTextureBindGroups {
    values: HashMap<Handle<Image>, BindGroup>,
}

pub fn prepare_shape_texture_bind_groups(
    render_device: Res<RenderDevice>,
    shape_pipelines: Res<ShapePipelines>,
    batches: Query<&ShapePipelineMaterial>,
    gpu_images: Res<RenderAssets<Image>>,
    mut image_bind_groups: ResMut<ShapeTextureBindGroups>,
) {
    for material in &batches {
        if let Some(handle) = &material.texture {
            if let Some(gpu_image) = gpu_images.get(handle.clone_weak()) {
                image_bind_groups
                    .values
                    .entry(handle.clone_weak())
                    .or_insert_with(|| {
                        render_device.create_bind_group(
                            "shape_texture_bind_group",
                            &shape_pipelines.texture_layout,
                            &BindGroupEntries::sequential((
                                &gpu_image.texture_view,
                                &gpu_image.sampler,
                            )),
                        )
                    });
            }
        }
    }
}

#[derive(Resource)]
pub struct ShapeBindGroup<T: ShapeData> {
    pub value: BindGroup,
    _marker: PhantomData<T>,
}

pub fn prepare_shape_bind_group<T: ShapeData + 'static>(
    mut commands: Commands,
    pipeline: Res<ShapePipeline<T>>,
    render_device: Res<RenderDevice>,
    shape_buffer: Res<GpuArrayBuffer<T>>,
) {
    if let Some(binding) = shape_buffer.binding() {
        commands.insert_resource(ShapeBindGroup {
            value: render_device.create_bind_group(
                "shape_bind_group",
                &pipeline.layout,
                &BindGroupEntries::single(binding),
            ),
            _marker: PhantomData::<T>,
        });
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

pub struct SetShapeTextureBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetShapeTextureBindGroup<I> {
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<ShapePipelineMaterial>;
    type Param = SRes<ShapeTextureBindGroups>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        material: &'w ShapePipelineMaterial,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(handle) = &material.texture {
            let bind_groups = bind_groups.into_inner();
            pass.set_bind_group(
                I,
                bind_groups.values.get(&handle.clone_weak()).unwrap(),
                &[],
            );
        }
        RenderCommandResult::Success
    }
}

pub struct SetShapeBindGroup<T: ShapeData, const I: usize>(PhantomData<T>);

impl<const I: usize, T: ShapeData + 'static, P: PhaseItem> RenderCommand<P>
    for SetShapeBindGroup<T, I>
{
    type Param = SRes<ShapeBindGroup<T>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: (),
        shape_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mut dynamic_offsets: [u32; 1] = Default::default();
        let mut offset_count = 0;
        if let Some(dynamic_offset) = item.dynamic_offset() {
            dynamic_offsets[offset_count] = dynamic_offset.get();
            offset_count += 1;
        }
        pass.set_bind_group(
            I,
            &shape_bind_group.into_inner().value,
            &dynamic_offsets[..offset_count],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawShape<T: ShapeData>(PhantomData<T>);

impl<P: PhaseItem, T: ShapeData> RenderCommand<P> for DrawShape<T> {
    type Param = SRes<QuadVertices>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: (),
        quad: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let batch_range = item.batch_range();
        #[cfg(all(feature = "webgl", target_arch = "wasm32"))]
        pass.set_push_constants(
            ShaderStages::VERTEX,
            0,
            &(batch_range.start as i32).to_le_bytes(),
        );
        pass.set_vertex_buffer(0, quad.into_inner().buffer.slice(..));
        pass.draw(0..T::VERTICES, batch_range.clone());

        RenderCommandResult::Success
    }
}
