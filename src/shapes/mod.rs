use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    prelude::*,
    render::{RenderLayersHash, ShapePipelineType},
};

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
    pub alpha_mode: AlphaMode,
    /// Forcibly disable local anti-aliasing.
    pub disable_laa: bool,
    /// Target pipeline draw the shape.
    pub pipeline: ShapePipelineType,
    /// [`Canvas`] to draw the shape to.
    pub canvas: Option<Entity>,
    /// Texture to apply to the shape.
    pub texture: Option<Handle<Image>>,
    /// Render layers to use when rendering.
    pub render_layers: RenderLayersHash,
}

impl Default for ShapeMaterial {
    fn default() -> Self {
        Self {
            alpha_mode: AlphaMode::Blend,
            disable_laa: false,
            pipeline: ShapePipelineType::Shape2d,
            texture: None,
            canvas: None,
            render_layers: RenderLayersHash(RenderLayers::default()),
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
                render_layers: RenderLayersHash(config.render_layers.unwrap_or_default()),
            },
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
