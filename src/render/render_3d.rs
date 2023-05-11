use bevy::{
    core_pipeline::core_3d::*,
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

use crate::{painter::ShapeStorage, render::*, shapes::Shape3d, ShapePipelineType};

pub fn extract_shapes_3d<T: ShapeData>(
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
            With<Shape3d>,
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

    if let Some(iter) = storage.get::<T>(ShapePipelineType::Shape3d) {
        instances.extend(iter.cloned());
    }

    if !instances.is_empty() {
        commands.spawn((ShapeInstances::<T>(instances), Shape3d));
    }
}

type WithPhases = (
    With<RenderPhase<Opaque3d>>,
    With<RenderPhase<Transparent3d>>,
    With<RenderPhase<AlphaMask3d>>,
);

fn spawn_buffers<T: ShapeData>(
    commands: &mut Commands,
    render_device: &RenderDevice,
    view_entity: Entity,
    view: &ExtractedView,
    material: ShapePipelineMaterial,
    instances: &mut Vec<T>,
) {
    let rangefinder = view.rangefinder3d();
    instances.sort_by_cached_key(|i| FloatOrd(rangefinder.distance(&i.transform())));

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
            distance: rangefinder.distance(&instances[0].transform()),
            length: instances.len(),
        },
        ShapeType::<T>::default(),
        Shape3d,
    ));
}

fn compute_visibility<T: ShapeData>(
    commands: &mut Commands,
    render_device: &RenderDevice,
    views: &Query<(Entity, &ExtractedView, Option<&RenderLayers>), WithPhases>,
    material: &ShapePipelineMaterial,
    mut instances: Vec<T>,
) {
    if instances.is_empty() {
        return;
    }

    debug_assert!(
        material.pipeline == ShapePipelineType::Shape3d,
        "Attempting to draw 2D shape in 3D pipeline. Ensure you are setting config.pipeline correctly."
    );

    for (view_entity, view, render_layers) in views {
        let render_layers = render_layers.cloned().unwrap_or_default();
        if !render_layers.intersects(&material.render_layers) {
            continue;
        }

        spawn_buffers(
            commands,
            render_device,
            view_entity,
            view,
            material.clone(),
            &mut instances,
        )
    }
}

pub fn prepare_shape_buffers_3d<T: ShapeData>(
    mut commands: Commands,
    mut query: Query<&mut ShapeInstances<T>, With<Shape3d>>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedView, Option<&RenderLayers>), WithPhases>,
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
pub fn queue_shapes_3d<T: ShapeData>(
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    alpha_mask_draw_functions: Res<DrawFunctions<AlphaMask3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<ShapePipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    shape_buffers: Query<(Entity, &ShapeDataBuffer), (With<ShapeType<T>>, With<Shape3d>)>,
    mut shape_pipelines: ResMut<ShapePipelines>,
    mut views: Query<(
        &ExtractedView,
        &mut RenderPhase<Opaque3d>,
        &mut RenderPhase<AlphaMask3d>,
        &mut RenderPhase<Transparent3d>,
    )>,
) where
    T: 'static,
{
    let draw_opaque = opaque_draw_functions.read().id::<DrawShapeCommand>();
    let draw_alpha_mask = alpha_mask_draw_functions.read().id::<DrawShapeCommand>();
    let draw_transparent = transparent_draw_functions.read().id::<DrawShapeCommand>();

    for (entity, buffer) in &shape_buffers {
        let (view, mut opaque_phase, mut alpha_mask_phase, mut transparent_phase) = views
            .get_mut(buffer.view)
            .expect("View entity is gone during queue instances, oh no!");

        let mut key = ShapePipelineKey::from_msaa_samples(msaa.samples());
        key |= ShapePipelineKey::from_hdr(view.hdr);
        key |= ShapePipelineKey::from_material(&buffer.material);

        if !buffer.material.disable_laa {
            key |= ShapePipelineKey::LOCAL_AA;
        }

        let pipeline = shape_pipelines.specialize::<T>(&pipeline_cache, pipeline.as_ref(), key);
        match buffer.material.alpha_mode.0 {
            AlphaMode::Opaque => {
                opaque_phase.add(Opaque3d {
                    entity,
                    draw_function: draw_opaque,
                    pipeline,
                    distance: buffer.distance,
                });
            }
            AlphaMode::Mask(_) => {
                alpha_mask_phase.add(AlphaMask3d {
                    entity,
                    draw_function: draw_alpha_mask,
                    pipeline,
                    distance: buffer.distance,
                });
            }
            AlphaMode::Blend | AlphaMode::Premultiplied | AlphaMode::Add | AlphaMode::Multiply => {
                transparent_phase.add(Transparent3d {
                    entity,
                    draw_function: draw_transparent,
                    pipeline,
                    distance: buffer.distance,
                });
            }
        }
    }
}
