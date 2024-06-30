use crate::{painter::ShapeStorage, render::*, shapes::Shape3d};
use bevy::{
    ecs::entity::EntityHashMap,
    render::{
        render_phase::{DrawFunctions, PhaseItemExtraIndex},
        render_resource::*,
        view::{ExtractedView, RenderLayers},
        Extract,
    },
    utils::HashMap,
};

#[derive(Resource, Deref, DerefMut)]
pub struct Shape2dInstances<T: ShapeData>(EntityHashMap<ShapeInstance<T>>);

impl<T: ShapeData> Default for Shape2dInstances<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Shape2dMaterials<T: ShapeData>(
    #[deref] HashMap<ShapePipelineMaterial, Vec<Entity>>,
    PhantomData<T>,
);

impl<T: ShapeData> Default for Shape2dMaterials<T> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

pub fn extract_shapes_2d<T: ShapeData>(
    mut commands: Commands,
    entities: Extract<
        Query<
            (
                Entity,
                &T::Component,
                &ShapeFill,
                &GlobalTransform,
                &InheritedVisibility,
                Option<&ShapeMaterial>,
                Option<&RenderLayers>,
            ),
            Without<Shape3d>,
        >,
    >,
    storage: Extract<Res<ShapeStorage>>,
    mut instance_data: ResMut<Shape2dInstances<T>>,
    mut materials: ResMut<Shape2dMaterials<T>>,
) {
    instance_data.clear();
    materials.clear();

    entities
        .iter()
        .filter_map(|(e, cp, fill, tf, vis, flags, rl)| {
            if vis.get() {
                Some((
                    e,
                    ShapePipelineMaterial::new(flags, rl),
                    cp.get_data(tf, fill),
                ))
            } else {
                None
            }
        })
        .for_each(|(entity, material, data)| {
            materials.entry(material.clone()).or_default().push(entity);
            instance_data.insert(entity, (material, data));
        });

    if let Some(iter) = storage.get::<T>(ShapePipelineType::Shape2d) {
        iter.cloned().for_each(|(material, data)| {
            let entity = commands.spawn_empty().id();
            materials.entry(material.clone()).or_default().push(entity);
            instance_data.insert(entity, (material, data));
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_shapes_2d<T: ShapeData>(
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<Shape2dPipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    materials: Res<Shape2dMaterials<T>>,
    instance_data: Res<Shape2dInstances<T>>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut views: Query<(Entity, &ExtractedView, Option<&RenderLayers>)>,
) {
    let draw_function = transparent_2d_draw_functions
        .read()
        .id::<DrawShape2dCommand<T>>();
    let view_count = views.iter().count();

    for (material, entities) in materials.iter() {
        let mut key = ShapePipelineKey::from_material(material);
        if !material.disable_laa {
            key |= ShapePipelineKey::LOCAL_AA;
        }

        let mut visible_views = Vec::with_capacity(view_count);
        if let Some(canvas) = material.canvas {
            if let Ok(view) = views.get_mut(canvas) {
                visible_views.push(view);
            }
        } else {
            views
                .iter_mut()
                .filter(|(_, _, layers)| {
                    let render_layers = layers.cloned().unwrap_or_default();
                    render_layers.intersects(&material.render_layers.0)
                })
                .for_each(|view| visible_views.push(view))
        };

        for (view_entity, view, _) in visible_views.into_iter() {
            let Some(transparent_phase) = phases.get_mut(&view_entity) else {
                continue;
            };

            let mut view_key = key;
            view_key |= ShapePipelineKey::from_msaa_samples(msaa.samples());
            view_key |= ShapePipelineKey::from_hdr(view.hdr);
            view_key |= ShapePipelineKey::PIPELINE_2D;
            let pipeline = shape_pipelines.specialize(&pipeline_cache, pipeline.as_ref(), view_key);

            for entity in entities {
                // SAFETY: we insert this alongside inserting into the vector we are currently iterating
                let (_, data) = unsafe { instance_data.get(entity).unwrap_unchecked() };
                transparent_phase.add(Transparent2d {
                    entity: *entity,
                    pipeline,
                    draw_function,
                    sort_key: FloatOrd(data.distance()),
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::NONE,
                });
            }
        }
    }
}

#[derive(Resource)]
pub struct Shape2dBindGroup<T: ShapeData> {
    pub value: BindGroup,
    _marker: PhantomData<T>,
}

pub fn prepare_shape_2d_bind_group<T: ShapeData + 'static>(
    mut commands: Commands,
    pipeline: Res<Shape2dPipeline<T>>,
    render_device: Res<RenderDevice>,
    shape_buffer: Res<BatchedInstanceBuffer<T>>,
) {
    if let Some(binding) = shape_buffer.binding() {
        commands.insert_resource(Shape2dBindGroup {
            value: render_device.create_bind_group(
                "shape_bind_group",
                &pipeline.layout,
                &BindGroupEntries::single(binding),
            ),
            _marker: PhantomData::<T>,
        });
    }
}
