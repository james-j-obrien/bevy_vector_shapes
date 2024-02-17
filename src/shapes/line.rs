use bevy::{
    prelude::*,
    reflect::Reflect,
    render::render_resource::{ShaderRef, ShaderType},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, LINE_HANDLE},
};

/// Component containing the data for drawing a line.
#[derive(Component, Reflect)]
pub struct LineComponent {
    pub alignment: Alignment,
    pub cap: Cap,

    /// Position to draw the start of the line in world space relative to it's transform.
    pub start: Vec3,
    /// Position to draw the end of the line in world space relative to it's transform.
    pub end: Vec3,
}

impl LineComponent {
    pub fn new(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        Self {
            alignment: config.alignment,
            cap: config.cap,

            start,
            end,
        }
    }
}

impl Default for LineComponent {
    fn default() -> Self {
        Self {
            alignment: default(),
            cap: default(),

            start: default(),
            end: default(),
        }
    }
}

impl ShapeComponent for LineComponent {
    type Data = LineData;

    fn get_data(&self, tf: &GlobalTransform, fill: &ShapeFill) -> LineData {
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
        flags.set_cap(self.cap);

        LineData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: fill.color.as_linear_rgba_f32(),
            thickness,
            flags: flags.0,

            start: self.start,
            end: self.end,
        }
    }
}

/// Raw data sent to the line shader to draw a line
#[derive(Clone, Copy, Reflect, Default, Debug, ShaderType)]
#[repr(C)]
pub struct LineData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    start: Vec3,
    end: Vec3,
}

impl LineData {
    pub fn new(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_cap(config.cap);

        LineData {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            start,
            end,
        }
    }
}

impl ShapeData for LineData {
    type Component = LineComponent;

    fn vertex_layout() -> Vec<wgpu::VertexAttribute> {
        vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,

            4 => Float32x4,
            5 => Float32,
            6 => Uint32,
            7 => Float32x3,
            8 => Float32x3,
        ]
        .to_vec()
    }

    fn shader() -> ShaderRef {
        LINE_HANDLE.into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw lines.
pub trait LinePainter {
    fn line(&mut self, start: Vec3, end: Vec3) -> &mut Self;
}

impl<'w, 's> LinePainter for ShapePainter<'w, 's> {
    fn line(&mut self, start: Vec3, end: Vec3) -> &mut Self {
        self.send(LineData::new(self.config(), start, end))
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of line bundles.
pub trait LineBundle {
    fn line(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self;
}

impl LineBundle for ShapeBundle<LineComponent> {
    fn line(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        let mut bundle = Self::new(config, LineComponent::new(config, start, end));
        bundle.fill.ty = FillType::Stroke(config.thickness, config.thickness_type);
        bundle
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of line entities.
pub trait LineSpawner<'w>: ShapeSpawner<'w> {
    fn line(&mut self, start: Vec3, end: Vec3) -> ShapeEntityCommands;
}

impl<'w, T: ShapeSpawner<'w>> LineSpawner<'w> for T {
    fn line(&mut self, start: Vec3, end: Vec3) -> ShapeEntityCommands {
        self.spawn_shape(ShapeBundle::line(self.config(), start, end))
    }
}
