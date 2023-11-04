use crate::{painter::ShapeStorage, render::*, shapes::Shape3d};
use bevy::{
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::*,
        view::{ExtractedView, RenderLayers},
        Extract,
    },
    utils::{EntityHashMap, FloatOrd, HashMap},
};

#[derive(Resource, Deref, DerefMut)]
pub struct Shape2dInstances<T: ShapeData>(EntityHashMap<Entity, ShapeInstance<T>>);

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
        .filter_map(|(e, cp, tf, vis, flags, rl)| {
            if vis.get() {
                Some((e, ShapePipelineMaterial::new(flags, rl), cp.get_data(tf)))
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
    pipeline: Res<ShapePipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    materials: Res<Shape2dMaterials<T>>,
    instance_data: Res<Shape2dInstances<T>>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut views: Query<(
        &ExtractedView,
        Option<&RenderLayers>,
        &mut RenderPhase<Transparent2d>,
    )>,
) {
    let draw_function = transparent_2d_draw_functions
        .read()
        .id::<DrawShapeCommand<T>>();
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
                .filter(|(_, layers, _)| {
                    let render_layers = layers.cloned().unwrap_or_default();
                    render_layers.intersects(&material.render_layers.0)
                })
                .for_each(|view| visible_views.push(view))
        };

        for (view, _, mut transparent_phase) in visible_views.into_iter() {
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
                    dynamic_offset: None,
                });
            }
        }
    }
}
