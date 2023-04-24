use bevy::prelude::*;

use crate::ShapeConfig;

mod disc;
pub use disc::*;

mod line;
pub use line::*;

mod rectangle;
pub use rectangle::*;

mod regular_polygon;
pub use regular_polygon::*;

/// Component that holds data related to a shape that is not consumed by it's shader,
#[derive(Component, Copy, Clone)]
pub struct Shape {
    /// Alpha mode to use when rendering, Opaque, Blend, Add and Multiply are explicitly supported.
    pub alpha_mode: AlphaMode,
    /// Forcibly disable local anti-aliasing.
    pub disable_laa: bool,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::Blend,
            disable_laa: false,
        }
    }
}

/// Bundle that is required to render a shape.
///
/// Shape specific methods will additionally add the component representing the corresponding shape.
#[derive(Bundle)]
pub struct ShapeBundle<T: Component> {
    pub spatial_bundle: SpatialBundle,
    pub shape: Shape,
    pub shape_type: T,
}

impl<T: Component> ShapeBundle<T> {
    pub fn new(config: &ShapeConfig, component: T) -> Self {
        Self {
            spatial_bundle: SpatialBundle::from_transform(config.transform),
            shape: Shape {
                alpha_mode: config.alpha_mode,
                disable_laa: config.disable_laa,
            },
            shape_type: component,
        }
    }
}

impl ShapeBundle<Line> {
    pub fn line(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        Self::new(config, Line::new(config, start, end))
    }
}

impl ShapeBundle<Rectangle> {
    pub fn rect(config: &ShapeConfig, size: Vec2) -> ShapeBundle<Rectangle> {
        ShapeBundle::<Rectangle>::new(config, Rectangle::new(config, size))
    }
}

impl ShapeBundle<RegularPolygon> {
    pub fn ngon(config: &ShapeConfig, sides: f32, radius: f32) -> ShapeBundle<RegularPolygon> {
        ShapeBundle::<RegularPolygon>::new(config, RegularPolygon::new(config, sides, radius))
    }
}

impl ShapeBundle<Disc> {
    pub fn circle(config: &ShapeConfig, radius: f32) -> ShapeBundle<Disc> {
        ShapeBundle::<Disc>::new(config, Disc::circle(config, radius))
    }

    pub fn arc(
        config: &ShapeConfig,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> ShapeBundle<Disc> {
        ShapeBundle::<Disc>::new(config, Disc::arc(config, radius, start_angle, end_angle))
    }
}

/// Defines the way in which the thickness value of shape is interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect, FromReflect)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect, FromReflect)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect, FromReflect)]
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
