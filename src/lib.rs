#![allow(clippy::type_complexity)]

use bevy::{prelude::*, render::camera::CameraUpdateSystem};

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
        BuildShapeChildren, Canvas, CanvasCommands, CanvasConfig, ShapeChildBuilder, ShapeCommands,
        ShapeConfig, ShapeEntityCommands, ShapePainter, ShapeSpawner,
    };
    pub use crate::{shapes::*, BaseShapeConfig, Shape2dPlugin, ShapePlugin};
}

/// Resource that represents the default shape config to be used by [`ShapePainter`]s.
///
/// When a [`ShapePainter`] is cleared it will have it's config reset to the current value of this resource.
#[derive(Resource, Copy, Clone)]
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
        load_shaders(app);
        app.init_resource::<ShapeStorage>()
            .insert_resource(BaseShapeConfig(self.base_config))
            .add_system(
                update_canvases
                    .in_base_set(CoreSet::PostUpdate)
                    .before(CameraUpdateSystem),
            )
            .add_system(clear_storage.in_base_set(CoreSet::PreUpdate))
            .add_plugin(InstancePlugin::<LineData>::default())
            .add_plugin(InstancePlugin::<RectData>::default())
            .add_plugin(InstancePlugin::<DiscData>::default())
            .add_plugin(InstancePlugin::<NgonData>::default());
    }
}

#[derive(Resource, Copy, Clone, Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum ShapeMode {
    Shape3d,
    Shape2d,
}

/// Plugin that contains all necessary functionality to draw shapes with a 3D or 2D camera.
pub struct ShapePlugin {
    /// Default config that will be used for all [`ShapePainter`]s.
    ///
    /// Available as a resource [`BaseShapeConfig`].
    pub base_config: ShapeConfig,
    /// Whether to also add the base plugin.
    ///
    /// Useful if you want to add the 3d functionality when another plugin has already added the base plugin.
    pub exclude_base: bool,
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
            exclude_base: false,
        }
    }
}

impl Plugin for ShapePlugin {
    fn build(&self, app: &mut App) {
        if !self.exclude_base {
            app.add_plugin(Shape2dPlugin::new(self.base_config.clone()));
        }
        app.add_plugin(Instance3dPlugin::<LineData>::default())
            .add_plugin(Instance3dPlugin::<RectData>::default())
            .add_plugin(Instance3dPlugin::<DiscData>::default())
            .add_plugin(Instance3dPlugin::<NgonData>::default());
    }
}
