use crate::prelude::*;
use bevy::prelude::*;

mod config;
pub use config::*;

mod system_param;
pub use system_param::*;

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
