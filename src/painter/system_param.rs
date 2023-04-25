use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{prelude::*, Immediate};

#[derive(Deref, DerefMut)]
struct LocalShapeConfig(pub ShapeConfig);

impl FromWorld for LocalShapeConfig {
    fn from_world(world: &mut World) -> Self {
        let config = world
            .get_resource::<BaseShapeConfig>()
            .cloned()
            .unwrap_or_default();

        Self(config.0)
    }
}

/// A system param that allows ergonomic spawning of shape entities.
///
/// The ShapeConfig used is initially extracted from the BaseShapeConfig resource.
/// Subsequent calls to .clear() will reset the config back to whatever is currently stored within the BaseShapeConfig resource.
///
/// Shapes will be spawned with commands during the next instance of [`apply_system_buffers`]
#[derive(SystemParam)]
pub struct ShapePainter<'w, 's> {
    config: Local<'s, LocalShapeConfig>,
    commands: Commands<'w, 's>,
    default_config: Res<'w, BaseShapeConfig>,
}

impl<'w, 's> ShapePainter<'w, 's> {
    pub fn set_config(&mut self, config: &ShapeConfig) {
        self.config.0 = *config;
    }

    /// Set the painter's [`ShapeConfig`] to the current value of the [`BaseShapeConfig`] resource.
    pub fn clear(&mut self) {
        self.config.0 = self.default_config.0;
    }
}

impl<'w, 's> ShapeSpawner<'w, 's> for ShapePainter<'w, 's> {
    fn config(&self) -> &ShapeConfig {
        &self.config.0
    }

    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands<'w, 's, '_> {
        let immediate = self.immediate;
        let Self {
            commands, config, ..
        } = self;
        let mut e = commands.spawn(bundle);
        if let Some(layers) = config.render_layers {
            e.insert(layers);
        }
        if immediate {
            e.insert(Immediate);
        }
        ShapeEntityCommands {
            commands: e,
            config,
        }
    }
}

impl<'w, 's> Deref for ShapePainter<'w, 's> {
    type Target = ShapeConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<'w, 's> DerefMut for ShapePainter<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}
