use crate::{painter::ShapeStorage, render::*, shapes::Shape3d};
use bevy::{
    ecs::entity::hash_map::EntityHashMap,
    render::{
        render_phase::{DrawFunctions, PhaseItemExtraIndex},
        render_resource::*,
        sync_world::RenderEntity,
        view::ExtractedView,
        Extract,
    },
};

#[derive(Resource, Deref, DerefMut)]
pub struct Shape2dInstanceBuffer<T: ShapeData>(ShapeInstanceBuffer<T>);

impl<T: ShapeData> Default for Shape2dInstanceBuffer<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: ShapeData> ShapeBatchSource<Transparent2d> for Shape2dInstanceBuffer<T> {
    type Data = T;
    type Command = DrawShape2dCommand<T>;
}

pub fn extract_shapes_2d<T: ShapeData>(
    shapes: Extract<
        Query<
            (
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
    mut buffer: ResMut<Shape2dInstanceBuffer<T>>,
    render_entities: Extract<Query<&RenderEntity>>,
    mut canvases: Local<EntityHashMap<Entity>>,
) {
    buffer.clear();
    canvases.clear();

    for (cp, fill, tf, vis, flags, rl) in shapes.iter() {
        if !vis.get() {
            continue;
        }
        let material = buffer.intern(&ShapePipelineMaterial::new(flags, rl));
        let data = cp.get_data(tf, fill);
        // Cache the sort key in the origin so queueing doesn't recompute it per view.
        buffer.push(material, Vec3::new(0.0, 0.0, data.distance()), data);
    }

    if let Some(iter) = storage.get::<T>(ShapePipelineType::Shape2d) {
        for instance in iter {
            // Map canvas entities
            let material = if instance.material.canvas.is_some() {
                let mut material = instance.material.clone();
                if let Some(canvas) = &mut material.canvas {
                    *canvas = *canvases.entry(*canvas).or_insert_with(|| {
                        render_entities
                            .get(*canvas)
                            .map(|e| e.id())
                            .unwrap_or(Entity::PLACEHOLDER)
                    });
                }
                buffer.intern(&material)
            } else {
                buffer.intern(&instance.material)
            };
            let data = instance.data.clone();
            buffer.push(material, Vec3::new(0.0, 0.0, data.distance()), data);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_shapes_2d<T: ShapeData>(
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<Shape2dPipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    buffer: Res<Shape2dInstanceBuffer<T>>,
    tag: Res<ShapeTypeTag<T>>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    views: Query<(&ExtractedView, &Msaa, Option<&RenderLayers>)>,
) {
    let draw_function = transparent_2d_draw_functions
        .read()
        .id::<DrawShape2dCommand<T>>();
    let default_layers = RenderLayers::default();
    let mut visible_views = Vec::with_capacity(views.iter().count());

    for (material_index, material) in buffer.materials.iter().enumerate() {
        let indices = &buffer.groups[material_index];
        if indices.is_empty() {
            continue;
        }

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
            let Some(transparent_phase) = phases.get_mut(&view.retained_view_entity) else {
                continue;
            };

            let mut view_key = key;
            view_key |= ShapePipelineKey::from_msaa_samples(msaa.samples());
            view_key |= ShapePipelineKey::PIPELINE_2D;
            let pipeline = shape_pipelines.specialize(
                &pipeline_cache,
                pipeline.as_ref(),
                view_key,
                view.target_format,
            );

            for &index in indices {
                transparent_phase.add_transient(Transparent2d {
                    entity: tag.phase_key(index),
                    pipeline,
                    draw_function,
                    sort_key: FloatOrd(buffer.origins[index as usize].z),
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::None,
                    extracted_index: index as usize,
                    indexed: true,
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

        commands.insert_resource(Shape2dBindGroup {
            value,
            _marker: PhantomData::<T>,
        });
    }
}
