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
        render_resource::{BindGroup, *},
        renderer::RenderDevice,
        texture::GpuImage,
        view::{ExtractedView, ViewUniformOffset, ViewUniforms},
    },
    utils::HashMap,
};

use crate::render::*;

pub type DrawShape2dCommand<T> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetShape2dBindGroup<T, 1>,
    SetShapeTextureBindGroup<2>,
    DrawShape<T>,
);

pub type DrawShape3dCommand<T> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetShape3dBindGroup<T, 1>,
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
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut image_bind_groups: ResMut<ShapeTextureBindGroups>,
) {
    for material in &batches {
        if let Some(handle) = &material.texture {
            if let Some(gpu_image) = gpu_images.get(handle.id()) {
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

pub struct SetShapeViewBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetShapeViewBindGroup<I> {
    type ViewQuery = (Read<ViewUniformOffset>, Read<ShapeViewBindGroup>);
    type ItemQuery = ();
    type Param = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        (view_uniform, shape_view_bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<()>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &shape_view_bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

pub struct SetShapeTextureBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetShapeTextureBindGroup<I> {
    type ViewQuery = ();
    type ItemQuery = Read<ShapePipelineMaterial>;
    type Param = SRes<ShapeTextureBindGroups>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        material: Option<&'w ShapePipelineMaterial>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(material) = material else {
            return RenderCommandResult::Success;
        };
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

pub struct SetShape2dBindGroup<T: ShapeData, const I: usize>(PhantomData<T>);

impl<const I: usize, T: ShapeData + 'static, P: PhaseItem> RenderCommand<P>
    for SetShape2dBindGroup<T, I>
{
    type Param = SRes<Shape2dBindGroup<T>>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        shape_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mut dynamic_offsets: [u32; 1] = Default::default();
        let mut offset_count = 0;
        if let Some(dynamic_offset) = item.extra_index().as_dynamic_offset() {
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

pub struct SetShape3dBindGroup<T: ShapeData, const I: usize>(PhantomData<T>);

impl<const I: usize, T: ShapeData + 'static, P: PhaseItem> RenderCommand<P>
    for SetShape3dBindGroup<T, I>
{
    type Param = SRes<Shape3dBindGroup<T>>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        shape_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mut dynamic_offsets: [u32; 1] = Default::default();
        let mut offset_count = 0;
        if let Some(dynamic_offset) = item.extra_index().as_dynamic_offset() {
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
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
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
