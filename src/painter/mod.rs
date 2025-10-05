use std::ops::DerefMut;

use bevy::{camera::CameraUpdateSystems, prelude::*};

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

    /// Note: [`ShapeBundle`](crate::ShapeBundle) does not include [`RenderLayers`](bevy::render::view::RenderLayers) as there is no support for optional components
    /// so instead it is inserted in this function conditionally depending on the [`ShapeConfig`] in `self`
    /// Prefer the function for the shape you want over [`ShapeSpawner::spawn_shape`], e.g. `commands.rect(...)`
    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands;
}

/// Plugin that setups up resources and systems for [`Canvas`] and [`ShapePainter`].
pub struct PainterPlugin;

impl Plugin for PainterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShapeStorage>()
            .add_systems(First, clear_storage)
            .add_systems(PostUpdate, update_canvases.before(CameraUpdateSystems));
    }
}
