use std::ops::DerefMut;

use bevy::{prelude::*, render::camera::CameraUpdateSystem};

mod config;
pub use config::*;

mod shape_commands;
pub use shape_commands::*;

mod child_commands;
pub use child_commands::*;

mod shape_painter;
pub use shape_painter::*;

mod canvas;
pub use canvas::*;

/// Trait that contains logic for spawning shape entities by type.
///
/// Implemented by [`ShapeCommands`] and [`ShapeChildBuilder`].
pub trait ShapeSpawner<'w>: DerefMut<Target = ShapeConfig> {
    fn config(&self) -> &ShapeConfig;

    fn set_config(&mut self, config: ShapeConfig);

    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands;
}

/// Plugin that setups up resources and systems for [`Canvas`] and [`ShapePainter`].
pub struct PainterPlugin;

impl Plugin for PainterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeStorage>()
            .add_systems(First, clear_storage)
            .add_systems(PostUpdate, update_canvases.before(CameraUpdateSystem));
    }
}
