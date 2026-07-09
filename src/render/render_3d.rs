use bevy::{
    core_pipeline::core_3d::*,
    ecs::entity::hash_map::EntityHashMap,
    platform::collections::HashMap,
    prelude::*,
    render::{
        render_phase::DrawFunctions,
        render_resource::*,
        sync_world::{MainEntity, RenderEntity},
        view::ExtractedView,
        Extract,
    },
};

use crate::{painter::ShapeStorage, render::*, shapes::Shape3d};

#[derive(Resource, Deref, DerefMut)]
pub struct Shape3dInstances<T: ShapeData>(EntityHashMap<(ShapeMaterialIndex, ShapeInstance<T>)>);

impl<T: ShapeData> Default for Shape3dInstances<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Shape3dMaterials<T: ShapeData>(
    #[deref] HashMap<ShapePipelineMaterial, (ShapeMaterialIndex, Vec<Entity>)>,
    PhantomData<T>,
);

impl<T: ShapeData> Default for Shape3dMaterials<T> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn extract_shapes_3d<T: ShapeData>(
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
    tag: Res<ShapeTypeTag<T>>,
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
            // Only clone the material when it hasn't been seen before this frame
            let material_index =
                if let Some((index, entities)) = materials.get_mut(&instance.material) {
                    entities.push(entity);
                    *index
                } else {
                    let index = ShapeMaterialIndex(materials.len() as u32);
                    materials.insert(instance.material.clone(), (index, vec![entity]));
                    index
                };
            instance_data.insert(entity, (material_index, instance));
        });

    if let Some(iter) = storage.get::<T>(ShapePipelineType::Shape3d) {
        for (index, mut instance) in iter.cloned().enumerate() {
            let entity = tag.fabricate(index as u32);
            if let Some(canvas) = &mut instance.material.canvas {
                *canvas = *canvases.entry(*canvas).or_insert_with(|| {
                    render_entities
                        .get(*canvas)
                        .map(|e| e.id())
                        .unwrap_or(Entity::PLACEHOLDER)
                });
            }
            let material_index =
                if let Some((index, entities)) = materials.get_mut(&instance.material) {
                    entities.push(entity);
                    *index
                } else {
                    let index = ShapeMaterialIndex(materials.len() as u32);
                    materials.insert(instance.material.clone(), (index, vec![entity]));
                    index
                };
            instance_data.insert(entity, (material_index, instance));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_shapes_3d<T: ShapeData>(
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<Shape3dPipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    materials: Res<Shape3dMaterials<T>>,
    instance_data: Res<Shape3dInstances<T>>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut trans_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa, Option<&RenderLayers>)>,
) {
    let draw_transparent = transparent_draw_functions
        .read()
        .id::<DrawShape3dCommand<T>>();
    let default_layers = RenderLayers::default();
    let mut visible_views = Vec::with_capacity(views.iter().count());

    for (material, (_, entities)) in materials.iter() {
        let mut key = ShapePipelineKey::from_material(material);
        if !material.disable_laa {
            key |= ShapePipelineKey::LOCAL_AA;
        }

        visible_views.clear();
        if let Some(canvas) = material.canvas {
            if let Ok(view) = views.get(canvas) {
                visible_views.push(view);
            }
        } else {
            views
                .iter()
                .filter(|(_, _, layers)| {
                    layers
                        .unwrap_or(&default_layers)
                        .intersects(&material.render_layers.0)
                })
                .for_each(|view| visible_views.push(view))
        };

        for &(view, msaa, _) in visible_views.iter() {
            let Some(transparent_phase) = trans_phases.get_mut(&view.retained_view_entity) else {
                continue;
            };
            let mut view_key = key;
            view_key |= ShapePipelineKey::from_msaa_samples(msaa.samples());
            let pipeline = shape_pipelines.specialize(
                &pipeline_cache,
                pipeline.as_ref(),
                view_key,
                view.target_format,
            );

            let rangefinder = view.rangefinder3d();
            for &entity in entities {
                // SAFETY: we insert this alongside inserting into the vector we are currently iterating
                let (_, instance) = unsafe { instance_data.get(&entity).unwrap_unchecked() };
                let distance = rangefinder.distance(&instance.origin);
                transparent_phase.add_transient(Transparent3d {
                    entity: (entity, MainEntity::from(entity)),
                    draw_function: draw_transparent,
                    pipeline,
                    sorting_info: TransparentSortingInfo3d::Sorted {
                        mesh_center: instance.origin,
                        depth_bias: 0.0,
                    },
                    distance,
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::None,
                    indexed: true,
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
    mut layout: Local<Option<BindGroupLayout>>,
    mut cached: Local<Option<(wgpu::Buffer, BindGroup)>>,
) {
    if let Some(binding) = shape_buffer.binding() {
        // The bind group only depends on the underlying buffer, which is reused
        // between frames unless it needed to grow, so cache by buffer identity.
        let cache_hit = match (&binding, cached.as_ref()) {
            (BindingResource::Buffer(new), Some((buffer, _))) => new.buffer == buffer,
            _ => false,
        };

        let value = if cache_hit {
            cached.as_ref().unwrap().1.clone()
        } else {
            let bind_group_layout = layout.get_or_insert_with(|| {
                render_device
                    .create_bind_group_layout("shape_bind_group_layout", &pipeline.layout.entries)
            });
            let buffer = match &binding {
                BindingResource::Buffer(new) => Some(new.buffer.clone()),
                _ => None,
            };
            let bind_group = render_device.create_bind_group(
                "shape_bind_group",
                bind_group_layout,
                &BindGroupEntries::single(binding),
            );
            if let Some(buffer) = buffer {
                *cached = Some((buffer, bind_group.clone()));
            }
            bind_group
        };

        commands.insert_resource(Shape3dBindGroup {
            value,
            _marker: PhantomData::<T>,
        });
    }
}
