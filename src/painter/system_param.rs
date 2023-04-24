use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

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
    pub fn config(&self) -> &ShapeConfig {
        &self.config.0
    }

    pub fn set_config(&mut self, config: &ShapeConfig) {
        self.config.0 = *config;
    }

    /// Set the painter's [`ShapeConfig`] to the current value of the [`BaseShapeConfig`] resource.
    pub fn clear(&mut self) {
        self.config.0 = self.default_config.0;
    }

    fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'w, 's, '_> {
        let immediate = self.immediate;
        let mut commands = self.commands.spawn(bundle);
        if let Some(layers) = self.config.render_layers {
            commands.insert(layers);
        }
        if immediate {
            commands.insert(Immediate);
        }
        commands
    }

    pub fn line(&mut self, start: Vec3, end: Vec3) -> EntityCommands<'w, 's, '_> {
        self.spawn(ShapeBundle::line(&self.config.0, start, end))
    }

    pub fn rect(&mut self, size: Vec2) -> EntityCommands<'w, 's, '_> {
        self.spawn(ShapeBundle::rect(&self.config.0, size))
    }

    pub fn ngon(&mut self, sides: f32, radius: f32) -> EntityCommands<'w, 's, '_> {
        self.spawn(ShapeBundle::ngon(&self.config.0, sides, radius))
    }

    pub fn circle(&mut self, radius: f32) -> EntityCommands<'w, 's, '_> {
        self.spawn(ShapeBundle::circle(&self.config.0, radius))
    }

    pub fn arc(
        &mut self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> EntityCommands<'w, 's, '_> {
        self.spawn(ShapeBundle::arc(
            &self.config.0,
            radius,
            start_angle,
            end_angle,
        ))
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
