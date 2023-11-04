use bevy::{
    prelude::*,
    reflect::Reflect,
    render::render_resource::{ShaderRef, ShaderType},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, RECT_HANDLE},
};

/// Component containing the data for drawing a rectangle.
#[derive(Component, Reflect)]
pub struct Rectangle {
    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    pub hollow: bool,

    /// Size of the rectangle on the x and y axis.
    pub size: Vec2,
    /// Corner rounding radius for each corner in world units.
    pub corner_radii: Vec4,
}

impl Rectangle {
    pub fn new(config: &ShapeConfig, size: Vec2) -> Self {
        Self {
            color: config.color,
            thickness: config.thickness,
            thickness_type: config.thickness_type,
            alignment: config.alignment,
            hollow: config.hollow,

            size,
            corner_radii: config.corner_radii,
        }
    }
}

impl ShapeComponent for Rectangle {
    type Data = RectData;

    fn get_data(&self, tf: &GlobalTransform) -> RectData {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);

        RectData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_linear_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            size: self.size.into(),
            corner_radii: self.corner_radii.into(),
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            thickness: 1.0,
            thickness_type: default(),
            alignment: default(),
            hollow: false,

            size: Vec2::ONE,
            corner_radii: default(),
        }
    }
}

/// Raw data sent to the rectangle shader to draw a rectangle
#[derive(Clone, Copy, Reflect, Default, Debug, ShaderType)]
#[repr(C)]
pub struct RectData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    size: [f32; 2],
    corner_radii: [f32; 4],
}

impl RectData {
    pub fn new(config: &ShapeConfig, size: Vec2) -> Self {
        let mut flags = Flags(0);
        flags.set_alignment(config.alignment);
        flags.set_thickness_type(config.thickness_type);
        flags.set_hollow(config.hollow as u32);

        Self {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            size: size.into(),
            corner_radii: config.corner_radii.into(),
        }
    }
}

impl ShapeData for RectData {
    type Component = Rectangle;

    fn vertex_layout() -> Vec<wgpu::VertexAttribute> {
        vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,

            4 => Float32x4,
            5 => Float32,
            6 => Uint32,
            7 => Float32x2,
            8 => Float32x4
        ]
        .to_vec()
    }

    fn shader() -> ShaderRef {
        RECT_HANDLE.into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw rectangles.
pub trait RectPainter {
    fn rect(&mut self, size: Vec2) -> &mut Self;

    fn image(&mut self, image: Handle<Image>, size: Vec2) -> &mut Self;
}

impl<'w, 's> RectPainter for ShapePainter<'w, 's> {
    fn rect(&mut self, size: Vec2) -> &mut Self {
        self.send(RectData::new(self.config(), size))
    }

    fn image(&mut self, image: Handle<Image>, size: Vec2) -> &mut Self {
        let mut config = self.config().clone();
        config.texture = Some(image);
        config.color = Color::WHITE;
        config.hollow = false;
        self.send_with_config(&config, RectData::new(&config, size))
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of rectangle bundles.
pub trait RectangleBundle {
    fn rect(config: &ShapeConfig, size: Vec2) -> Self;
}

impl RectangleBundle for ShapeBundle<Rectangle> {
    fn rect(config: &ShapeConfig, size: Vec2) -> Self {
        Self::new(config, Rectangle::new(config, size))
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of rectangle entities.
pub trait RectangleSpawner<'w, 's> {
    fn rect(&mut self, size: Vec2) -> ShapeEntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: ShapeSpawner<'w, 's>> RectangleSpawner<'w, 's> for T {
    fn rect(&mut self, size: Vec2) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::rect(self.config(), size))
    }
}
