use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    render::{setup_pipeline_2d, setup_pipeline_3d, setup_pipeline_common, ShapeData},
    ShapeConfig,
};

mod disc;
pub use disc::*;

mod line;
pub use line::*;

mod rectangle;
pub use rectangle::*;

mod regular_polygon;
pub use regular_polygon::*;

#[derive(Clone)]
pub enum ShapeFill {
    Color(Color),
    Texture(Handle<Image>),
}

/// Component that holds data related to a shape that is not consumed by it's shader,
#[derive(Component, Clone)]
pub struct ShapeMaterial {
    /// Alpha mode to use when rendering, Opaque, Blend, Add and Multiply are explicitly supported.
    pub alpha_mode: AlphaMode,
    /// Forcibly disable local anti-aliasing.
    pub disable_laa: bool,
    pub canvas: Option<Entity>,
    pub color: Color,
    pub fill: ShapeFill,
}

impl ShapeMaterial {
    pub fn set_color(&mut self, color: Color) {
        self.fill = Shape
    }
}

impl Default for ShapeMaterial {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::Blend,
            disable_laa: false,
            canvas: None,
            fill: ShapeFill::Color(Color::WHITE),
        }
    }
}

#[derive(Component)]
pub struct Shape3d;

/// Bundle that is required to render a shape.
///
/// Shape specific methods will additionally add the component representing the corresponding shape.
#[derive(Bundle)]
pub struct ShapeBundle<T: Component> {
    pub spatial_bundle: SpatialBundle,
    pub shape: ShapeMaterial,
    pub shape_type: T,
}

impl<T: Component> ShapeBundle<T> {
    pub fn new(config: &ShapeConfig, component: T) -> Self {
        Self {
            spatial_bundle: SpatialBundle::from_transform(config.transform),
            shape: ShapeMaterial {
                alpha_mode: config.alpha_mode,
                disable_laa: config.disable_laa,
                canvas: config.canvas,
            },
            shape_type: component,
        }
    }

    pub fn insert_3d(self) -> (Self, Shape3d) {
        (self, Shape3d)
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

#[derive(Default)]
pub(crate) struct InstancePlugin<T: ShapeData> {
    _marker: PhantomData<T>,
}

impl<T: ShapeData> Plugin for InstancePlugin<T> {
    fn build(&self, app: &mut App) {
        app.register_type::<T::Component>();
        setup_pipeline_common::<T>(app);
        setup_pipeline_2d::<T>(app);
    }
}

#[derive(Default)]
pub(crate) struct Instance3dPlugin<T: ShapeData> {
    _marker: PhantomData<T>,
}

impl<T: ShapeData> Plugin for Instance3dPlugin<T> {
    fn build(&self, app: &mut App) {
        setup_pipeline_3d::<T>(app);
    }
}
