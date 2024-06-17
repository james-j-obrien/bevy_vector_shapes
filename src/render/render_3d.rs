use bevy::{
    core_pipeline::core_3d::*,
    ecs::entity::EntityHashMap,
    prelude::*,
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::*,
        view::{ExtractedView, RenderLayers},
        Extract,
    },
    utils::HashMap,
};

use crate::{painter::ShapeStorage, render::*, shapes::Shape3d};

#[derive(Resource, Deref, DerefMut)]
pub struct Shape3dInstances<T: ShapeData>(EntityHashMap<ShapeInstance<T>>);

impl<T: ShapeData> Default for Shape3dInstances<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Shape3dMaterials<T: ShapeData>(
    #[deref] HashMap<ShapePipelineMaterial, Vec<Entity>>,
    PhantomData<T>,
);

impl<T: ShapeData> Default for Shape3dMaterials<T> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

pub fn extract_shapes_3d<T: ShapeData>(
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
                Option<&ShapeOrigin>,
            ),
            With<Shape3d>,
        >,
    >,
    storage: Extract<Res<ShapeStorage>>,
    mut instance_data: ResMut<Shape3dInstances<T>>,
    mut materials: ResMut<Shape3dMaterials<T>>,
) {
    instance_data.clear();
    materials.clear();

    entities
        .iter()
        .filter_map(|(e, cp, fill, tf, vis, flags, rl, or)| {
            if vis.get() {
                // find global origin of shape
                let local_origin = or.map(|or| or.0).unwrap_or(Vec3::ZERO);
                let origin = tf.transform_point(local_origin);

                Some((
                    e,
                    ShapeInstance {
                        material: ShapePipelineMaterial::new(flags, rl),
                        origin,
                        data: cp.get_data(tf, fill),
                    },
                ))
            } else {
                None
            }
        })
        .for_each(|(entity, instance)| {
            materials
                .entry(instance.material.clone())
                .or_default()
                .push(entity);
            instance_data.insert(entity, instance);
        });

    if let Some(iter) = storage.get::<T>(ShapePipelineType::Shape3d) {
        iter.cloned().for_each(|instance| {
            let entity = commands.spawn_empty().id();
            materials
                .entry(instance.material.clone())
                .or_default()
                .push(entity);
            instance_data.insert(entity, instance);
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_shapes_3d<T: ShapeData>(
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    alpha_mask_draw_functions: Res<DrawFunctions<AlphaMask3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<ShapePipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    materials: Res<Shape3dMaterials<T>>,
    instance_data: Res<Shape3dInstances<T>>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut views: Query<(
        &ExtractedView,
        Option<&RenderLayers>,
        &mut RenderPhase<Opaque3d>,
        &mut RenderPhase<AlphaMask3d>,
        &mut RenderPhase<Transparent3d>,
    )>,
) {
    let draw_opaque = opaque_draw_functions.read().id::<DrawShapeCommand<T>>();
    let draw_alpha_mask = alpha_mask_draw_functions.read().id::<DrawShapeCommand<T>>();
    let draw_transparent = transparent_draw_functions
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
                .filter(|(_, layers, ..)| {
                    let render_layers = layers.cloned().unwrap_or_default();
                    render_layers.intersects(&material.render_layers.0)
                })
                .for_each(|view| visible_views.push(view))
        };

        for (view, _, mut opaque_phase, mut alpha_mask_phase, mut transparent_phase) in
            visible_views.into_iter()
        {
            let mut view_key = key;
            view_key |= ShapePipelineKey::from_msaa_samples(msaa.samples());
            view_key |= ShapePipelineKey::from_hdr(view.hdr);
            let pipeline = shape_pipelines.specialize(&pipeline_cache, pipeline.as_ref(), view_key);

            let rangefinder = view.rangefinder3d();
            for &entity in entities {
                // SAFETY: we insert this alongside inserting into the vector we are currently iterating
                let instance = unsafe { instance_data.get(&entity).unwrap_unchecked() };
                let data = &instance.data;
                let dist_point = data.transform().transform_vector3(instance.origin);
                let distance = rangefinder.distance_translation(&dist_point);
                match material.alpha_mode.0 {
                    AlphaMode::Opaque => {
                        opaque_phase.add(Opaque3d {
                            asset_id: AssetId::Uuid {
                                uuid: AssetId::<Mesh>::DEFAULT_UUID,
                            },
                            entity,
                            draw_function: draw_opaque,
                            pipeline,
                            batch_range: 0..1,
                            dynamic_offset: None,
                        });
                    }
                    AlphaMode::Mask(_) => {
                        alpha_mask_phase.add(AlphaMask3d {
                            entity,
                            draw_function: draw_alpha_mask,
                            pipeline,
                            distance,
                            batch_range: 0..1,
                            dynamic_offset: None,
                        });
                    }
                    AlphaMode::Blend
                    | AlphaMode::Premultiplied
                    | AlphaMode::Add
                    | AlphaMode::Multiply => {
                        transparent_phase.add(Transparent3d {
                            entity,
                            draw_function: draw_transparent,
                            pipeline,
                            distance,
                            batch_range: 0..1,
                            dynamic_offset: None,
                        });
                    }
                }
            }
        }
    }
}
