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

use crate::render::*;

pub fn extract_instances<T: Instanceable>(
    mut commands: Commands,
    entities: Extract<
        Query<(
            &T::Component,
            &GlobalTransform,
            &ComputedVisibility,
            Option<&Shape>,
            Option<&RenderLayers>,
        )>,
    >,
    mut events: Extract<EventReader<ShapeEvent<T>>>,
) {
    let mut instances = entities
        .iter()
        .filter_map(|(cp, tf, vis, flags, rl)| {
            if vis.is_visible() {
                Some((RenderKey::new(flags, rl), cp.instance(tf)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    instances.extend(events.iter().map(|e| (e.render_key, e.instance)));

    if !instances.is_empty() {
        commands.spawn(InstanceData::<T>(instances));
    }
}

type WithPhases = (
    With<RenderPhase<Opaque3d>>,
    With<RenderPhase<Transparent3d>>,
    With<RenderPhase<AlphaMask3d>>,
);

fn spawn_buffers<T: Instanceable>(
    commands: &mut Commands,
    render_device: &RenderDevice,
    views: &Query<(Entity, &ExtractedView, Option<&RenderLayers>), WithPhases>,
    key: RenderKey,
    mut instances: Vec<T>,
) {
    if instances.is_empty() {
        return;
    }
    for (view_entity, view, render_layers) in views {
        let render_layers = render_layers.cloned().unwrap_or_default();
        if !render_layers.intersects(&key.render_layers) {
            continue;
        }

        let rangefinder = view.rangefinder3d();
        instances.sort_by_cached_key(|i| FloatOrd(rangefinder.distance(&i.transform())));

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instances.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.spawn(InstanceBuffer::<T> {
            view: view_entity,
            key,
            buffer,
            distance: rangefinder.distance(&instances[0].transform()),
            length: instances.len(),
            _marker: default(),
        });
    }
}

pub fn prepare_instance_buffers<T: Instanceable>(
    mut commands: Commands,
    mut query: Query<&mut InstanceData<T>>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedView, Option<&RenderLayers>), WithPhases>,
) {
    for mut instance_data in &mut query {
        instance_data.sort_by_key(|(k, _i)| *k);

        let (key, instances) = instance_data.iter().fold(
            (instance_data[0].0, Vec::new()),
            |(key, mut instances), (next_key, instance)| {
                if *next_key == key {
                    instances.push(*instance);
                    (key, instances)
                } else {
                    spawn_buffers(
                        &mut commands,
                        render_device.as_ref(),
                        &views,
                        key,
                        instances,
                    );

                    (*next_key, vec![*instance])
                }
            },
        );

        spawn_buffers(
            &mut commands,
            render_device.as_ref(),
            &views,
            key,
            instances,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_instances<T: Instanceable>(
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    alpha_mask_draw_functions: Res<DrawFunctions<AlphaMask3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<InstancedPipeline<T>>>,
    instanced_pipeline: ResMut<InstancedPipeline<T>>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    instance_buffers: Query<(Entity, &InstanceBuffer<T>)>,
    mut views: Query<(
        &ExtractedView,
        &mut RenderPhase<Opaque3d>,
        &mut RenderPhase<AlphaMask3d>,
        &mut RenderPhase<Transparent3d>,
    )>,
) where
    T: 'static,
{
    let draw_opaque = opaque_draw_functions.read().id::<DrawInstancedCommand<T>>();
    let draw_alpha_mask = alpha_mask_draw_functions
        .read()
        .id::<DrawInstancedCommand<T>>();
    let draw_transparent = transparent_draw_functions
        .read()
        .id::<DrawInstancedCommand<T>>();

    for (entity, buffer) in &instance_buffers {
        let (view, mut opaque_phase, mut alpha_mask_phase, mut transparent_phase) = views
            .get_mut(buffer.view)
            .expect("View entity is gone during queue instances, oh no!");

        let mut key = InstancedPipelineKey::from_msaa_samples(msaa.samples());
        key |= InstancedPipelineKey::from_hdr(view.hdr);
        key |= InstancedPipelineKey::from_alpha_mode(buffer.key.alpha_mode.0);

        if !buffer.key.disable_laa {
            key |= InstancedPipelineKey::LOCAL_AA;
        }

        let pipeline = pipelines.specialize(&pipeline_cache, &instanced_pipeline, key);
        match buffer.key.alpha_mode.0 {
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
