use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{prelude::*, render::ShapePipelineType};

/// A system param that allows ergonomic spawning of shape entities.
///
/// The [`ShapeConfig`] used is initially extracted from the [`BaseShapeConfig`] resource.
/// Subsequent calls to `reset()` will reset the config back to whatever is currently stored within the [`BaseShapeConfig`] resource.
///
/// Shapes will be spawned with commands during the next instance of [`apply_deferred`]
#[derive(SystemParam)]
pub struct ShapeCommands<'w, 's> {
    config: &'s mut ShapeConfig,
    commands: Commands<'w, 's>,
    default_config: Res<'w, BaseShapeConfig>,
}

impl ShapeCommands<'_, '_> {
    /// Set the painter's [`ShapeConfig`] to the current value of the [`BaseShapeConfig`] resource.
    pub fn reset(&mut self) {
        *self.config = self.default_config.0.clone();
    }
}

impl<'w> ShapeSpawner<'w> for ShapeCommands<'w, '_> {
    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands {
        let Self {
            commands, config, ..
        } = self;
        let mut entity = commands.spawn(bundle);
        if let Some(layers) = &config.render_layers {
            entity.insert(layers.clone());
        }
        if let ShapePipelineType::Shape3d = config.pipeline {
            entity.insert(Shape3d);
        }

        ShapeEntityCommands {
            commands: entity,
            config,
        }
    }

    fn config(&self) -> &ShapeConfig {
        self.config
    }

    fn set_config(&mut self, config: ShapeConfig) {
        *self.config = config;
    }
}

impl Deref for ShapeCommands<'_, '_> {
    type Target = ShapeConfig;

    fn deref(&self) -> &Self::Target {
        self.config
    }
}

impl DerefMut for ShapeCommands<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.config
    }
}
