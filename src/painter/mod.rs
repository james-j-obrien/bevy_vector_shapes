use crate::prelude::*;
use bevy::prelude::*;

mod config;
pub use config::*;

mod system_param;
pub use system_param::*;

mod child_painter;
pub use child_painter::*;

pub(crate) struct PainterPlugin;

impl Plugin for PainterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BaseShapeConfig>()
            .add_system(clear_immediate_shapes.in_base_set(CoreSet::PreUpdate));
    }
}

pub(crate) struct Painter2dPlugin;

impl Plugin for Painter2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BaseShapeConfig>()
            .add_system(clear_immediate_shapes.in_base_set(CoreSet::PreUpdate));
    }
}

/// Marker component attached to shapes spawned in immediate mode.
#[derive(Component)]
pub struct Immediate;

fn clear_immediate_shapes(mut commands: Commands, shapes: Query<Entity, With<Immediate>>) {
    shapes.for_each(|s| commands.entity(s).despawn());
}

/// Trait that contains logic for drawing each shape type.
///
/// Implemented by [`ShapePainter`] and [`ChildPainter`].
pub trait ShapeSpawner<'w, 's> {
    fn config(&self) -> &ShapeConfig;

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
