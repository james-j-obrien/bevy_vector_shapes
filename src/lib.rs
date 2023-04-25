#![allow(clippy::type_complexity)]

use bevy::prelude::*;

/// Components and Enums used to define shapes.
pub mod shapes;
use shapes::*;

/// Rendering specific traits and structs.
pub mod render;
use render::load_shaders;

/// Structs and components used by the [`ShapePainter`].
pub mod painter;
use painter::*;

/// `use bevy_vector_shapes::prelude::*` to import commonly used items.
pub mod prelude {
    pub use crate::painter::{
        ChildPainter, ShapeConfig, ShapeEntityCommands, ShapePainter, ShapeSpawner,
    };
    pub use crate::{
        shapes::{
            Alignment, Cap, Disc, Line, Rectangle, RegularPolygon, Shape, ShapeBundle,
            ThicknessType,
        },
        BaseShapeConfig, Shape2dPlugin, ShapePlugin,
    };
}

/// Resource that represents the default shape config to be used by [`ShapePainter`]s.
///
/// When a [`ShapePainter`] is cleared it will have it's config reset to the current value of this resource.
#[derive(Resource, Copy, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct BaseShapeConfig(pub ShapeConfig);

/// Plugin that contains all necessary functionality to draw shapes with a 3D camera.
#[derive(Default)]
pub struct ShapePlugin {
    /// Default config that will be used for all [`ShapePainter`]s.
    ///
    /// Available as a resource [`BaseShapeConfig`].
    pub base_config: ShapeConfig,
}

impl ShapePlugin {
    /// Creates a shape plugin where the default config is in retained mode.
    pub fn retained() -> Self {
        let mut s = Self::default();
        s.base_config.immediate = false;
        s
    }

    /// Creates a shape plugin where the default config is in immediate mode.
    pub fn immediate() -> Self {
        Self::default()
    }
}

impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        load_shaders(app);
        app.add_plugin(LinePlugin)
            .add_plugin(RectPlugin)
            .add_plugin(RegularPolygonPlugin)
            .add_plugin(DiscPlugin)
            .add_plugin(PainterPlugin)
            .insert_resource(BaseShapeConfig(self.base_config));
    }
}

/// Plugin that contains all necessary functionality to draw shapes with a 2D camera.
#[derive(Default)]
pub struct Shape2dPlugin {
    /// Default config that will be used for all [`ShapePainter`]s.
    ///
    /// Available as a resource [`BaseShapeConfig`].
    pub base_config: ShapeConfig,
}

impl Shape2dPlugin {
    /// Creates a shape plugin where the default config is in retained mode.
    pub fn retained() -> Self {
        let mut s = Self::default();
        s.base_config.immediate = false;
        s
    }

    /// Creates a shape plugin where the default config is in immediate mode.
    pub fn immediate() -> Self {
        Self::default()
    }
}

impl Plugin for Shape2dPlugin {
    fn build(&self, app: &mut App) {
        load_shaders(app);
        app.add_plugin(Line2dPlugin)
            .add_plugin(Rect2dPlugin)
            .add_plugin(RegularPolygon2dPlugin)
            .add_plugin(Disc2dPlugin)
            .add_plugin(Painter2dPlugin)
            .insert_resource(BaseShapeConfig(self.base_config));
    }
}
