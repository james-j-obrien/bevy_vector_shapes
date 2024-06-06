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
pub struct RectangleComponent {
    pub alignment: Alignment,

    /// Size of the rectangle on the x and y axis.
    pub size: Vec2,
    /// Corner rounding radius for each corner in world units.
    pub corner_radii: Vec4,
}

impl RectangleComponent {
    pub fn new(config: &ShapeConfig, size: Vec2) -> Self {
        Self {
            alignment: config.alignment,

            size,
            corner_radii: config.corner_radii,
        }
    }
}

impl ShapeComponent for RectangleComponent {
    type Data = RectData;

    fn get_data(&self, tf: &GlobalTransform, fill: &ShapeFill) -> RectData {
        let mut flags = Flags(0);
        let thickness = match fill.ty {
            FillType::Stroke(thickness, thickness_type) => {
                flags.set_thickness_type(thickness_type);
                flags.set_hollow(1);
                thickness
            }
            FillType::Fill => 1.0,
        };
        flags.set_alignment(self.alignment);

        RectData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: fill.color.linear().to_f32_array(),
            thickness,
            flags: flags.0,

            size: self.size.into(),
            corner_radii: self.corner_radii.into(),
        }
    }
}

impl Default for RectangleComponent {
    fn default() -> Self {
        Self {
            alignment: default(),

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

            color: config.color.linear().to_f32_array(),
            thickness: config.thickness,
            flags: flags.0,

            size: size.into(),
            corner_radii: config.corner_radii.into(),
        }
    }
}

impl ShapeData for RectData {
    type Component = RectangleComponent;

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

impl RectangleBundle for ShapeBundle<RectangleComponent> {
    fn rect(config: &ShapeConfig, size: Vec2) -> Self {
        Self::new(config, RectangleComponent::new(config, size))
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of rectangle entities.
pub trait RectangleSpawner<'w> {
    fn rect(&mut self, size: Vec2) -> ShapeEntityCommands;
}

impl<'w, T: ShapeSpawner<'w>> RectangleSpawner<'w> for T {
    fn rect(&mut self, size: Vec2) -> ShapeEntityCommands {
        self.spawn_shape(ShapeBundle::rect(self.config(), size))
    }
}
