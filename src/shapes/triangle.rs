use bevy::{
    math::vec2,
    prelude::*,
    reflect::Reflect,
    render::render_resource::{ShaderRef, ShaderType},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, TRIANGLE_HANDLE},
};

/// Component containing the data for drawing a triangle.
#[derive(Component, Reflect)]
pub struct Triangle {
    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    pub hollow: bool,
    pub vertices: [Vec2; 3],
    pub roundness: f32,
}

impl Triangle {
    pub fn new(config: &ShapeConfig, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> Self {
        Self {
            color: config.color,
            thickness: config.thickness,
            thickness_type: config.thickness_type,
            alignment: config.alignment,
            hollow: config.hollow,
            vertices: [v_a, v_b, v_c],
            roundness: config.roundness,
        }
    }
}

impl ShapeComponent for Triangle {
    type Data = TriangleData;

    fn get_data(&self, tf: &GlobalTransform) -> TriangleData {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);

        TriangleData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_linear_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            vertices: [
                self.vertices[0].into(),
                self.vertices[1].into(),
                self.vertices[2].into(),
            ],
            roundness: self.roundness,

            padding: default(),
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            thickness: 1.0,
            thickness_type: default(),
            alignment: default(),
            hollow: false,

            vertices: [vec2(0.5, 0.0), vec2(0.0, 0.7), vec2(-0.5, 0.0)],
            roundness: 0.0,
        }
    }
}

/// Raw data sent to the triangle shader to draw a triangle
#[derive(Clone, Copy, Reflect, Default, Debug, ShaderType)]
#[repr(C)]
pub struct TriangleData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    vertices: [[f32; 2]; 3],
    roundness: f32,

    padding: [f32; 3],
}

impl TriangleData {
    pub fn new(config: &ShapeConfig, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> TriangleData {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);

        TriangleData {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,
            vertices: [v_a.into(), v_b.into(), v_c.into()],
            roundness: config.roundness,

            padding: default(),
        }
    }
}

impl ShapeData for TriangleData {
    type Component = Triangle;
    const VERTICES: u32 = 3;

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
            8 => Float32x2,
            9 => Float32x2,
            10 => Float32,
        ]
        .to_vec()
    }

    fn shader() -> ShaderRef {
        TRIANGLE_HANDLE.into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw triangles.
pub trait TrianglePainter {
    fn triangle(&mut self, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> &mut Self;
}

impl<'w, 's> TrianglePainter for ShapePainter<'w, 's> {
    fn triangle(&mut self, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> &mut Self {
        self.send(TriangleData::new(self.config(), v_a, v_b, v_c))
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of triangle bundles.
pub trait TriangleBundle {
    fn triangle(config: &ShapeConfig, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> Self;
}

impl TriangleBundle for ShapeBundle<Triangle> {
    fn triangle(config: &ShapeConfig, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> Self {
        Self::new(config, Triangle::new(config, v_a, v_b, v_c))
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of triangle entities.
pub trait TriangleSpawner<'w, 's> {
    fn triangle(&mut self, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> ShapeEntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: ShapeSpawner<'w, 's>> TriangleSpawner<'w, 's> for T {
    fn triangle(&mut self, v_a: Vec2, v_b: Vec2, v_c: Vec2) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::triangle(self.config(), v_a, v_b, v_c))
    }
}
