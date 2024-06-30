use bevy::prelude::*;

use crate::{prelude::*, render::ShapePipelineType};

mod disc;
pub use disc::*;

mod line;
pub use line::*;

mod rectangle;
pub use rectangle::*;

mod regular_polygon;
pub use regular_polygon::*;

mod triangle;
pub use triangle::*;

/// Component that holds data related to a shape to be used during rendering,
#[derive(Component, Clone)]
pub struct ShapeMaterial {
    /// Alpha mode to use when rendering, Opaque, Blend, Add and Multiply are explicitly supported.
    pub alpha_mode: ShapeAlphaMode,
    /// Forcibly disable local anti-aliasing.
    pub disable_laa: bool,
    /// Target pipeline draw the shape.
    pub pipeline: ShapePipelineType,
    /// [`Canvas`] to draw the shape to.
    pub canvas: Option<Entity>,
    /// Texture to apply to the shape.
    pub texture: Option<Handle<Image>>,
}

impl Default for ShapeMaterial {
    fn default() -> Self {
        Self {
            alpha_mode: ShapeAlphaMode::Blend,
            disable_laa: false,
            pipeline: ShapePipelineType::Shape2d,
            texture: None,
            canvas: None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Reflect)]
pub enum ShapeAlphaMode {
    #[default]
    Blend,
    Add,
    Multiply,
}

impl From<AlphaMode> for ShapeAlphaMode {
    fn from(value: AlphaMode) -> Self {
        match value {
            AlphaMode::Add => ShapeAlphaMode::Add,
            AlphaMode::Multiply => ShapeAlphaMode::Multiply,
            _ => ShapeAlphaMode::Blend,
        }
    }
}

/// Used in [`ShapeFill`] to determine how a shape is rendered.
#[derive(Default, Clone, Copy)]
pub enum FillType {
    /// Fully colored shape
    #[default]
    Fill,
    /// An outline with the given thickness and [`ThicknessType`].
    ///
    /// Note: all [`LineComponent`] should use `Stroke`, otherwise they will default to thickness `1.0`.
    Stroke(f32, ThicknessType),
}

/// Component attached to each shape to determine how it is rendered.
#[derive(Default, Component, Clone, Copy)]
pub struct ShapeFill {
    pub color: Color,
    pub ty: FillType,
}

impl ShapeFill {
    pub fn new(config: &ShapeConfig) -> Self {
        Self {
            color: config.color,
            ty: if config.hollow {
                FillType::Stroke(config.thickness, config.thickness_type)
            } else {
                FillType::Fill
            },
        }
    }
}

/// Marker component for entities that should be drawn by the 3D pipeline.
#[derive(Component)]
pub struct Shape3d;

/// Bundle that is required to render a shape.
///
/// Shape specific methods will additionally add the component representing the corresponding shape.
#[derive(Bundle)]
pub struct ShapeBundle<T: Component> {
    pub spatial_bundle: SpatialBundle,
    pub shape: ShapeMaterial,
    pub fill: ShapeFill,
    pub shape_type: T,
}

impl<T: Component> ShapeBundle<T> {
    pub fn new(config: &ShapeConfig, component: T) -> Self {
        Self {
            spatial_bundle: SpatialBundle::from_transform(config.transform),
            shape: ShapeMaterial {
                alpha_mode: config.alpha_mode,
                disable_laa: config.disable_laa,
                pipeline: config.pipeline,
                canvas: config.canvas,
                texture: config.texture.clone(),
            },
            fill: ShapeFill::new(config),
            shape_type: component,
        }
    }

    /// Inserts the [`Shape3d`] marker component so that the entity is picked up by the associated pipeline.
    pub fn insert_3d(self) -> (Self, Shape3d) {
        (self, Shape3d)
    }
}

/// Defines the way in which the thickness value of shape is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum ThicknessType {
    /// 1.0 thickness corresponds to 1.0 world unit.
    #[default]
    World,
    /// 1.0 thickness corresponds to 1 pixel.
    Pixels,
    /// 1.0 thickness corresponds to 1% of the screen size along the shortest axis.
    Screen,
}

impl From<ThicknessType> for u32 {
    fn from(value: ThicknessType) -> Self {
        value as u32
    }
}

/// Defines the way in which caps will be rendered on a supported shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum Cap {
    /// No caps
    None,
    /// Lines with this cap will be extended by their thickness on each end
    Square,
    /// Lines or Discs with this cap will have semi-circles attached at each end
    #[default]
    Round,
}

impl From<Cap> for u32 {
    fn from(value: Cap) -> Self {
        value as u32
    }
}

/// Defines how a shape will orient itself in relation to it's transform and the camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum Alignment {
    /// Shapes will respect the rotation in their transform.
    #[default]
    Flat,
    /// Shapes will always orient themselves towards the camera.
    /// Note that lines rotate around their direction while all other shapes will fully face the camera at all times.
    Billboard,
}

impl From<Alignment> for u32 {
    fn from(value: Alignment) -> Self {
        value as u32
    }
}
