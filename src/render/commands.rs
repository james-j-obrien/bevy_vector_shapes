use bevy::{
    core_pipeline::tonemapping::{get_lut_bindings, Tonemapping, TonemappingLuts},
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    platform::collections::HashMap,
    prelude::*,
    render::{
        globals::GlobalsBuffer,
        render_asset::RenderAssets,
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{BindGroup, *},
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
        view::{ExtractedView, ViewUniformOffset, ViewUniforms},
    },
};

use crate::render::*;

pub type DrawShape2dCommand<T> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetShape2dBindGroup<T, 1>,
    SetShape2dTextureBindGroup<T, 2>,
    DrawShape<T>,
);

pub type DrawShape3dCommand<T> = (
    SetItemPipeline,
    SetShapeViewBindGroup<0>,
    SetShape3dBindGroup<T, 1>,
    SetShape3dTextureBindGroup<T, 2>,
    DrawShape<T>,
);

#[derive(Component, Debug)]
pub struct ShapeViewBindGroup {
    value: BindGroup,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_shape_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    shape_pipeline: Res<ShapePipelines>,
    view_uniforms: Res<ViewUniforms>,
    globals_buffer: Res<GlobalsBuffer>,
    views: Query<(Entity, &Tonemapping), With<ExtractedView>>,
    tonemapping_luts: Res<TonemappingLuts>,
    images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
    mut layout: Local<Option<BindGroupLayout>>,
) {
    let (Some(view_binding), Some(globals)) = (
        view_uniforms.uniforms.binding(),
        globals_buffer.buffer.binding(),
    ) else {
        return;
    };

    for (entity, tonemapping) in views.iter() {
        let view_bind_group_layout = layout.get_or_insert_with(|| {
            render_device.create_bind_group_layout(
                "shape_view_bind_group_layout",
                &shape_pipeline.view_layout.entries,
            )
        });

        let lut_bindings =
            get_lut_bindings(&images, &tonemapping_luts, tonemapping, &fallback_image);
        let view_bind_group = render_device.create_bind_group(
            "shape_view_bind_group",
            view_bind_group_layout,
            &BindGroupEntries::with_indices((
                (0, view_binding.clone()),
                (1, globals.clone()),
                (2, lut_bindings.0),
                (3, lut_bindings.1),
            )),
        );

        commands.entity(entity).insert(ShapeViewBindGroup {
            value: view_bind_group,
        });
    }
}

/// Bind groups for each texture in use by a shape material.
///
/// Keyed by asset id rather than handle so this map doesn't keep dropped images alive,
/// the view id is stored so bind groups are recreated when an image is reloaded.
#[derive(Resource, Default)]
pub struct ShapeTextureBindGroups {
    values: HashMap<AssetId<Image>, (TextureViewId, BindGroup)>,
}

/// Drops bind groups for textures whose images no longer exist.
pub fn prune_shape_texture_bind_groups(
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut image_bind_groups: ResMut<ShapeTextureBindGroups>,
) {
    image_bind_groups
        .values
        .retain(|id, _| gpu_images.get(*id).is_some());
}

fn prepare_texture_bind_group(
    render_device: &RenderDevice,
    layout: &BindGroupLayout,
    gpu_images: &RenderAssets<GpuImage>,
    image_bind_groups: &mut ShapeTextureBindGroups,
    handle: &Handle<Image>,
) {
    let Some(gpu_image) = gpu_images.get(handle.id()) else {
        return;
    };
    let view_id = gpu_image.texture_view.id();
    let stale = image_bind_groups
        .values
        .get(&handle.id())
        .is_none_or(|(id, _)| *id != view_id);
    if stale {
        let bind_group = render_device.create_bind_group(
            "shape_texture_bind_group",
            layout,
            &BindGroupEntries::sequential((&gpu_image.texture_view, &gpu_image.sampler)),
        );
        image_bind_groups
            .values
            .insert(handle.id(), (view_id, bind_group));
    }
}

pub fn prepare_shape_2d_texture_bind_groups<T: ShapeData>(
    render_device: Res<RenderDevice>,
    shape_pipelines: Res<ShapePipelines>,
    materials: ResMut<Shape2dMaterials<T>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut image_bind_groups: ResMut<ShapeTextureBindGroups>,
    mut layout: Local<Option<BindGroupLayout>>,
) {
    let texture_bind_group_layout = layout.get_or_insert_with(|| {
        render_device.create_bind_group_layout(
            "shape_texture_bind_group_layout",
            &shape_pipelines.texture_layout.entries,
        )
    });
    for material in materials.keys() {
        if let Some(handle) = &material.texture {
            prepare_texture_bind_group(
                &render_device,
                texture_bind_group_layout,
                &gpu_images,
                &mut image_bind_groups,
                handle,
            );
        }
    }
}

pub fn prepare_shape_3d_texture_bind_groups<T: ShapeData>(
    render_device: Res<RenderDevice>,
    shape_pipelines: Res<ShapePipelines>,
    materials: ResMut<Shape3dMaterials<T>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut image_bind_groups: ResMut<ShapeTextureBindGroups>,
    mut layout: Local<Option<BindGroupLayout>>,
) {
    let texture_bind_group_layout = layout.get_or_insert_with(|| {
        render_device.create_bind_group_layout(
            "shape_texture_bind_group_layout",
            &shape_pipelines.texture_layout.entries,
        )
    });
    for material in materials.keys() {
        if let Some(handle) = &material.texture {
            prepare_texture_bind_group(
                &render_device,
                texture_bind_group_layout,
                &gpu_images,
                &mut image_bind_groups,
                handle,
            );
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
        (view_uniform, shape_view_bind_group): ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<()>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &shape_view_bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

pub struct SetShape2dTextureBindGroup<T: ShapeData, const I: usize>(PhantomData<T>);

impl<const I: usize, T: ShapeData, P: PhaseItem> RenderCommand<P>
    for SetShape2dTextureBindGroup<T, I>
{
    type ViewQuery = ();
    type ItemQuery = ();
    type Param = (SRes<ShapeTextureBindGroups>, SRes<Shape2dInstances<T>>);

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (bind_groups, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(material) = instances
            .get(&item.main_entity().id())
            .map(|(_, i)| &i.material)
        else {
            return RenderCommandResult::Success;
        };
        if let Some(handle) = &material.texture {
            let bind_groups = bind_groups.into_inner();
            let (_, bind_group) = bind_groups.values.get(&handle.id()).unwrap();
            pass.set_bind_group(I, bind_group, &[]);
        }
        RenderCommandResult::Success
    }
}

pub struct SetShape3dTextureBindGroup<T: ShapeData, const I: usize>(PhantomData<T>);

impl<const I: usize, T: ShapeData, P: PhaseItem> RenderCommand<P>
    for SetShape3dTextureBindGroup<T, I>
{
    type ViewQuery = ();
    type ItemQuery = ();
    type Param = (SRes<ShapeTextureBindGroups>, SRes<Shape3dInstances<T>>);

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        (bind_groups, instances): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(material) = instances
            .get(&item.main_entity().id())
            .map(|(_, i)| &i.material)
        else {
            return RenderCommandResult::Success;
        };
        if let Some(handle) = &material.texture {
            let bind_groups = bind_groups.into_inner();
            let (_, bind_group) = bind_groups.values.get(&handle.id()).unwrap();
            pass.set_bind_group(I, bind_group, &[]);
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
        if let PhaseItemExtraIndex::DynamicOffset(dynamic_offset) = item.extra_index() {
            dynamic_offsets[offset_count] = dynamic_offset;
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
        if let PhaseItemExtraIndex::DynamicOffset(dynamic_offset) = item.extra_index() {
            dynamic_offsets[offset_count] = dynamic_offset;
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
        let quad = quad.into_inner();
        pass.set_vertex_buffer(0, quad.buffer.slice(..));
        pass.set_index_buffer(quad.indices.slice(..), IndexFormat::Uint16);
        pass.draw_indexed(0..T::VERTICES, 0, batch_range.clone());

        RenderCommandResult::Success
    }
}
