use bevy::{
    core_pipeline::core_2d::*,
    prelude::*,
    render::{
        render_phase::{DrawFunctions, RenderPhase},
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, RenderLayers},
        Extract,
    },
    utils::FloatOrd,
};

use crate::{painter::ShapeStorage, render::*, shapes::Shape3d};

pub fn extract_shapes_2d<T: ShapeData>(
    mut commands: Commands,
    entities: Extract<
        Query<
            (
                &T::Component,
                &GlobalTransform,
                &ComputedVisibility,
                Option<&ShapeMaterial>,
                Option<&RenderLayers>,
            ),
            Without<Shape3d>,
        >,
    >,
    storage: Extract<Res<ShapeStorage>>,
) {
    let mut instances = entities
        .iter()
        .filter_map(|(cp, tf, vis, flags, rl)| {
            if vis.is_visible() {
                Some((ShapePipelineMaterial::new(flags, rl), cp.into_data(tf)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if let Some(iter) = storage.get::<T>(ShapePipelineType::Shape2d) {
        instances.extend(iter.cloned());
    }

    if !instances.is_empty() {
        commands.spawn(ShapeInstances::<T>(instances));
    }
}

fn spawn_buffers<T: ShapeData>(
    commands: &mut Commands,
    render_device: &RenderDevice,
    view_entity: Entity,
    material: ShapePipelineMaterial,
    instances: &mut Vec<T>,
) {
    instances.sort_by_cached_key(|i| FloatOrd(i.distance()));

    // Workaround for an issue in the implementation of Chromes webgl ANGLE D3D11 backend
    #[cfg(target_arch = "wasm32")]
    if instances.len() == 1 {
        instances.push(T::zeroed());
    }

    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("shape_instance_data_buffer"),
        contents: bytemuck::cast_slice(instances.as_slice()),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });
    commands.spawn((
        ShapeDataBuffer {
            view: view_entity,
            material,
            buffer,
            distance: instances[0].distance(),
            length: instances.len(),
        },
        ShapeType::<T>::default(),
    ));
}

fn compute_visibility<T: ShapeData>(
    commands: &mut Commands,
    render_device: &RenderDevice,
    views: &Query<
        (Entity, Option<&RenderLayers>),
        (With<ExtractedView>, With<RenderPhase<Transparent2d>>),
    >,
    material: &ShapePipelineMaterial,
    mut instances: Vec<T>,
) {
    if instances.is_empty() {
        return;
    }

    debug_assert!(
        material.pipeline == ShapePipelineType::Shape2d,
        "Attempting to draw 3D shape in 2D pipeline. Ensure you have the Shape3d component inserted."
    );

    if let Some(canvas) = material.canvas {
        if let Ok((view_entity, _)) = views.get(canvas) {
            spawn_buffers(
                commands,
                render_device,
                view_entity,
                material.clone(),
                &mut instances,
            );
        }
    } else {
        for (view_entity, render_layers) in views {
            let render_layers = render_layers.cloned().unwrap_or_default();
            if !render_layers.intersects(&material.render_layers) {
                continue;
            }

            spawn_buffers(
                commands,
                render_device,
                view_entity,
                material.clone(),
                &mut instances,
            );
        }
    }
}

pub fn prepare_shape_buffers_2d<T: ShapeData>(
    mut commands: Commands,
    mut query: Query<&mut ShapeInstances<T>, Without<Shape3d>>,
    render_device: Res<RenderDevice>,
    views: Query<
        (Entity, Option<&RenderLayers>),
        (With<ExtractedView>, With<RenderPhase<Transparent2d>>),
    >,
) {
    for mut instance_data in &mut query {
        instance_data.sort_by(|(a, _), (b, _)| a.cmp(b));

        let (key, instances) = instance_data.iter().fold(
            (&instance_data[0].0, Vec::new()),
            |(key, mut instances), (next_key, instance)| {
                if next_key == key {
                    instances.push(*instance);
                    (key, instances)
                } else {
                    compute_visibility(
                        &mut commands,
                        render_device.as_ref(),
                        &views,
                        key,
                        instances,
                    );

                    (next_key, vec![*instance])
                }
            },
        );

        compute_visibility(
            &mut commands,
            render_device.as_ref(),
            &views,
            key,
            instances,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_shapes_2d<T: ShapeData>(
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<ShapePipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    instance_buffers: Query<(Entity, &ShapeDataBuffer), (With<ShapeType<T>>, Without<Shape3d>)>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent2d>)>,
) {
    let draw_function = transparent_2d_draw_functions
        .read()
        .id::<DrawShapeCommand>();

    for (entity, buffer) in &instance_buffers {
        let (view, mut transparent_phase) = views
            .get_mut(buffer.view)
            .expect("View entity is gone during queue instances, oh no!");

        let mut key = ShapePipelineKey::from_msaa_samples(msaa.samples());
        key |= ShapePipelineKey::from_hdr(view.hdr);
        key |= ShapePipelineKey::PIPELINE_2D;
        key |= ShapePipelineKey::from_material(&buffer.material);

        if !buffer.material.disable_laa {
            key |= ShapePipelineKey::LOCAL_AA;
        }

        let pipeline = shape_pipelines.specialize(&pipeline_cache, pipeline.as_ref(), key);
        transparent_phase.add(Transparent2d {
            entity,
            pipeline,
            draw_function,
            sort_key: FloatOrd(buffer.distance),
            batch_range: None,
        });
    }
}
