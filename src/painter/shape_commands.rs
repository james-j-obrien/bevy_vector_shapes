use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{painter::LocalShapeConfig, prelude::*, ShapePipelineType};

/// A system param that allows ergonomic spawning of shape entities.
///
/// The ShapeConfig used is initially extracted from the [`BaseShapeConfig`] resource.
/// Subsequent calls to .clear() will reset the config back to whatever is currently stored within the [`BaseShapeConfig`] resource.
///
/// Shapes will be spawned with commands during the next instance of [`apply_system_buffers`]
#[derive(SystemParam)]
pub struct ShapeCommands<'w, 's> {
    config: Local<'s, LocalShapeConfig>,
    commands: Commands<'w, 's>,
    default_config: Res<'w, BaseShapeConfig>,
}

impl<'w, 's> ShapeCommands<'w, 's> {
    /// Set the painter's [`ShapeConfig`] to the current value of the [`BaseShapeConfig`] resource.
    pub fn clear(&mut self) {
        self.config.0 = self.default_config.0.clone();
    }
}

impl<'w, 's, 'a> ShapeSpawner<'w, 's> for ShapeCommands<'w, 's> {
    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands<'w, 's, '_> {
        let Self {
            commands, config, ..
        } = self;
        let mut e = commands.spawn(bundle);
        if let Some(layers) = config.render_layers {
            e.insert(layers);
        }
        if let ShapePipelineType::Shape3d = config.pipeline {
            e.insert(Shape3d);
        }

        ShapeEntityCommands {
            commands: e,
            config,
        }
    }

    fn config(&self) -> &ShapeConfig {
        &self.config.0
    }

    fn set_config(&mut self, config: ShapeConfig) {
        self.config.0 = config;
    }
}

impl<'w, 's> Deref for ShapeCommands<'w, 's> {
    type Target = ShapeConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<'w, 's> DerefMut for ShapeCommands<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}
