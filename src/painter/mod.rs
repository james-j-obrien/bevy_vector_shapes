use std::ops::DerefMut;

use crate::prelude::*;
use bevy::prelude::*;

mod config;
pub use config::*;

mod shape_commands;
pub use shape_commands::*;

mod child_commands;
pub use child_commands::*;

mod shape_painter;
pub use shape_painter::*;

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

/// Trait that contains logic for spawning shape entities by type.
///
/// Implemented by [`ShapeCommands`] and [`ShapeChildBuilder`].
pub trait ShapeSpawner<'w, 's>: DerefMut<Target = ShapeConfig> {
    fn config(&self) -> &ShapeConfig;

    fn set_config(&mut self, config: &ShapeConfig);

    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands<'w, 's, '_>;

    fn line(&mut self, start: Vec3, end: Vec3) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::line(self.config(), start, end))
    }

    fn rect(&mut self, size: Vec2) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::rect(self.config(), size))
    }

    fn ngon(&mut self, sides: f32, radius: f32) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::ngon(self.config(), sides, radius))
    }

    fn circle(&mut self, radius: f32) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::circle(self.config(), radius))
    }

    fn arc(
        &mut self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::arc(
            self.config(),
            radius,
            start_angle,
            end_angle,
        ))
    }
}
