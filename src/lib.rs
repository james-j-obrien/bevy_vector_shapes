#![allow(clippy::type_complexity)]

//! `bevy_vector_shapes` is a library for easily and ergonomically creating instanced vector shapes in [Bevy](https://bevyengine.org/).
//!
//! ## Usage
//! See the the [examples](https://github.com/james-j-obrien/bevy_vector_shapes/tree/main/examples) for more details on all supported features.
//! ```rust
//! use bevy::prelude::*;
//! // Import commonly used items
//! use bevy_vector_shapes::prelude::*;

//! fn main() {
//!     App::new()
//!         .add_plugins((
//!             DefaultPlugins,
//!             // Add the shape plugin:
//!             // - Shape2dPlugin for 2D cameras
//!             // - ShapePlugin for both 3D and 2D cameras
//!             Shape2dPlugin::default(),
//!         ))
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, draw)
//!         .run();
//! }

//! fn setup(mut commands: Commands) {
//!     // Spawn the camera
//!     commands.spawn(Camera2d);
//! }

//! fn draw(mut painter: ShapePainter) {
//!     // Draw a circle
//!     painter.circle(100.0);
//! }
//! ```
//!

use bevy::prelude::*;

/// Components and Enums used to define shape types.
pub mod shapes;
use shapes::*;

/// Rendering specific traits and structs.
pub mod render;
use render::{ShapeRenderPlugin, ShapeType3dPlugin, ShapeTypePlugin};

/// Structs and components used by the [`ShapePainter`], [`ShapeCommands`] and [`Canvas`] APIs.
pub mod painter;
use painter::*;

#[cfg(feature = "assets")]
pub mod asset;

/// `use bevy_vector_shapes::prelude::*` to import commonly used items.
pub mod prelude {
    pub use crate::painter::{
        BuildShapeChildren, Canvas, CanvasCommands, CanvasConfig, CanvasMode, ShapeChildBuilder,
        ShapeCommands, ShapeConfig, ShapeEntityCommands, ShapePainter, ShapeSpawner,
    };
    pub use crate::{shapes::*, BaseShapeConfig, Shape2dPlugin, ShapePlugin};
}

/// Resource that represents the default shape config to be used by [`ShapePainter`] and [`ShapeCommands`] APIs.
///
/// When a [`ShapePainter`] is cleared it will have it's config reset to the current value of this resource.
#[derive(Resource, Clone)]
pub struct BaseShapeConfig(pub ShapeConfig);

/// Plugin that contains all necessary functionality to draw shapes with a 2D camera.
pub struct Shape2dPlugin {
    /// Default config that will be used for all [`ShapePainter`]s.
    ///
    /// Available as a resource [`BaseShapeConfig`].
    pub base_config: ShapeConfig,
}

impl Default for Shape2dPlugin {
    fn default() -> Self {
        Self {
            base_config: ShapeConfig::default_2d(),
        }
    }
}

impl Shape2dPlugin {
    pub fn new(base_config: ShapeConfig) -> Self {
        Self { base_config }
    }
}

impl Plugin for Shape2dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BaseShapeConfig(self.base_config.clone()))
            .add_plugins(PainterPlugin)
            .add_plugins(ShapeRenderPlugin)
            .add_plugins(ShapeTypePlugin::<LineComponent>::default())
            .add_plugins(ShapeTypePlugin::<DiscComponent>::default())
            .add_plugins(ShapeTypePlugin::<RectangleComponent>::default())
            .add_plugins(ShapeTypePlugin::<RegularPolygonComponent>::default())
            .add_plugins(ShapeTypePlugin::<TriangleComponent>::default());
    }
}

/// Plugin that contains all necessary functionality to draw shapes with a 3D or 2D camera.
pub struct ShapePlugin {
    /// Default config that will be used for all [`ShapePainter`]s.
    ///
    /// Available as a resource [`BaseShapeConfig`].
    pub base_config: ShapeConfig,
    /// Whether to also add the 2d plugin.
    ///
    /// Useful if you want to add the 3d functionality when another plugin has already added the 2d plugin.
    pub exclude_2d: bool,
}

impl ShapePlugin {
    pub fn new(base_config: ShapeConfig) -> Self {
        Self {
            base_config,
            ..default()
        }
    }
}

impl Default for ShapePlugin {
    fn default() -> Self {
        Self {
            base_config: ShapeConfig::default_3d(),
            exclude_2d: false,
        }
    }
}

impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        if !self.exclude_2d {
            app.add_plugins(Shape2dPlugin::new(self.base_config.clone()));
        }
        app.add_plugins(ShapeType3dPlugin::<LineComponent>::default())
            .add_plugins(ShapeType3dPlugin::<DiscComponent>::default())
            .add_plugins(ShapeType3dPlugin::<RectangleComponent>::default())
            .add_plugins(ShapeType3dPlugin::<RegularPolygonComponent>::default())
            .add_plugins(ShapeType3dPlugin::<TriangleComponent>::default());
    }
}
