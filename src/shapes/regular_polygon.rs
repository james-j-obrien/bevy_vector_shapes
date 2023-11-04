use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    reflect::Reflect,
    render::render_resource::{ShaderRef, ShaderType},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, NGON_HANDLE},
};

/// Component containing the data for drawing a regular polygon.
#[derive(Component, Reflect)]
pub struct RegularPolygon {
    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    pub hollow: bool,

    /// Number of sides, non-integer values may have unexpected results.
    pub sides: f32,
    /// Radius to the tip of a corner.
    pub radius: f32,
    /// Corner rounding radius for all corner in world units.
    pub roundness: f32,
}

impl RegularPolygon {
    pub fn new(config: &ShapeConfig, sides: f32, radius: f32) -> Self {
        Self {
            color: config.color,
            thickness: config.thickness,
            thickness_type: config.thickness_type,
            alignment: config.alignment,
            hollow: config.hollow,

            sides,
            radius,
            roundness: config.roundness,
        }
    }
}

impl ShapeComponent for RegularPolygon {
    type Data = NgonData;

    fn get_data(&self, tf: &GlobalTransform) -> NgonData {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);

        NgonData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_linear_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            sides: self.sides,
            radius: self.radius,
            roundness: self.roundness,

            padding: default(),
        }
    }
}

impl Default for RegularPolygon {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            thickness: 1.0,
            thickness_type: default(),
            alignment: default(),
            hollow: false,

            sides: 3.0,
            radius: 1.0,
            roundness: 0.0,
        }
    }
}

/// Raw data sent to the regular polygon shader to draw a regular polygon
#[derive(Clone, Copy, Reflect, Pod, Zeroable, Default, Debug, ShaderType)]
#[repr(C)]
pub struct NgonData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    sides: f32,
    radius: f32,
    roundness: f32,

    padding: [f32; 3],
}

impl NgonData {
    pub fn new(config: &ShapeConfig, sides: f32, radius: f32) -> NgonData {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);

        NgonData {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            sides,
            radius,
            roundness: config.roundness,

            padding: default(),
        }
    }
}

impl ShapeData for NgonData {
    type Component = RegularPolygon;

    fn vertex_layout() -> Vec<wgpu::VertexAttribute> {
        vertex_attr_array![
            0 => Float32x4,
            1 => Float32x4,
            2 => Float32x4,
            3 => Float32x4,

            4 => Float32x4,
            5 => Float32,
            6 => Uint32,
            7 => Float32,
            8 => Float32,
            9 => Float32
        ]
        .to_vec()
    }

    fn shader() -> ShaderRef {
        NGON_HANDLE.into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw regular polygons.
pub trait RegularPolygonPainter {
    fn ngon(&mut self, sides: f32, radius: f32) -> &mut Self;
}

impl<'w, 's> RegularPolygonPainter for ShapePainter<'w, 's> {
    fn ngon(&mut self, sides: f32, radius: f32) -> &mut Self {
        self.send(NgonData::new(self.config(), sides, radius))
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of regular polygon bundles.
pub trait RegularPolygonBundle {
    fn ngon(config: &ShapeConfig, sides: f32, radius: f32) -> Self;
}

impl RegularPolygonBundle for ShapeBundle<RegularPolygon> {
    fn ngon(config: &ShapeConfig, sides: f32, radius: f32) -> Self {
        Self::new(config, RegularPolygon::new(config, sides, radius))
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of regular polygon entities.
pub trait RegularPolygonSpawner<'w, 's> {
    fn ngon(&mut self, sides: f32, radius: f32) -> ShapeEntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: ShapeSpawner<'w, 's>> RegularPolygonSpawner<'w, 's> for T {
    fn ngon(&mut self, sides: f32, radius: f32) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::ngon(self.config(), sides, radius))
    }
}
