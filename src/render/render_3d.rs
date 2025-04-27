use bevy::{
    core_pipeline::core_3d::*,
    ecs::entity::hash_map::EntityHashMap,
    platform::collections::HashMap,
    prelude::*,
    render::{
        render_phase::DrawFunctions,
        render_resource::*,
        sync_world::{MainEntity, RenderEntity, TemporaryRenderEntity},
        view::{ExtractedView, RenderLayers},
        Extract,
    },
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
    render_entities: Extract<Query<&RenderEntity>>,
    mut canvases: Local<EntityHashMap<Entity>>,
) {
    instance_data.clear();
    materials.clear();
    canvases.clear();

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
        iter.cloned().for_each(|mut instance| {
            let entity = commands.spawn(TemporaryRenderEntity).id();
            if let Some(canvas) = &mut instance.material.canvas {
                *canvas = *canvases.entry(*canvas).or_insert_with(|| {
                    render_entities
                        .get(*canvas)
                        .map(|e| e.id())
                        .unwrap_or(Entity::PLACEHOLDER)
                });
            }
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
    // opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    // alpha_mask_draw_functions: Res<DrawFunctions<AlphaMask3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<Shape3dPipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    materials: Res<Shape3dMaterials<T>>,
    instance_data: Res<Shape3dInstances<T>>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    // mut opaque_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    // mut alpha_phases: ResMut<ViewBinnedRenderPhases<AlphaMask3d>>,
    mut trans_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    mut views: Query<(&ExtractedView, &Msaa, Option<&RenderLayers>)>,
) {
    // let draw_opaque = opaque_draw_functions.read().id::<DrawShape3dCommand<T>>();
    // let draw_alpha_mask = alpha_mask_draw_functions
    //     .read()
    //     .id::<DrawShape3dCommand<T>>();
    let draw_transparent = transparent_draw_functions
        .read()
        .id::<DrawShape3dCommand<T>>();
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

        for (view, msaa, _) in visible_views.into_iter() {
            // let (Some(opaque_phase), Some(alpha_mask_phase), Some(transparent_phase)) = (
            //     opaque_phases.get_mut(&view_entity),
            //     alpha_phases.get_mut(&view_entity),
            //     trans_phases.get_mut(&view_entity),
            // ) else {
            //     continue;
            // };
            let Some(transparent_phase) = trans_phases.get_mut(&view.retained_view_entity) else {
                continue;
            };
            let mut view_key = key;
            view_key |= ShapePipelineKey::from_msaa_samples(msaa.samples());
            view_key |= ShapePipelineKey::from_hdr(view.hdr);
            let pipeline = shape_pipelines.specialize(&pipeline_cache, pipeline.as_ref(), view_key);

            // let default_id = AssetId::Uuid {
            //     uuid: AssetId::<Mesh>::DEFAULT_UUID,
            // };
            let rangefinder = view.rangefinder3d();
            for &entity in entities {
                // SAFETY: we insert this alongside inserting into the vector we are currently iterating
                let instance = unsafe { instance_data.get(&entity).unwrap_unchecked() };
                let distance = rangefinder.distance_translation(&instance.origin);
                transparent_phase.add(Transparent3d {
                    entity: (entity, MainEntity::from(Entity::PLACEHOLDER)),
                    draw_function: draw_transparent,
                    pipeline,
                    distance,
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::None,
                    indexed: false,
                });
            }
        }
    }
}

#[derive(Resource)]
pub struct Shape3dBindGroup<T: ShapeData> {
    pub value: BindGroup,
    _marker: PhantomData<T>,
}

pub fn prepare_shape_3d_bind_group<T: ShapeData + 'static>(
    mut commands: Commands,
    pipeline: Res<Shape3dPipeline<T>>,
    render_device: Res<RenderDevice>,
    shape_buffer: Res<BatchedInstanceBuffer<T>>,
) {
    if let Some(binding) = shape_buffer.binding() {
        commands.insert_resource(Shape3dBindGroup {
            value: render_device.create_bind_group(
                "shape_bind_group",
                &pipeline.layout,
                &BindGroupEntries::single(binding),
            ),
            _marker: PhantomData::<T>,
        });
    }
}
