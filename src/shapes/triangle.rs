use bevy::{
    core::{Pod, Zeroable},
    math::vec2,
    prelude::*,
    reflect::Reflect,
    render::render_resource::ShaderRef,
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
    pub fn new(config: &ShapeConfig, vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> Self {
        Self {
            color: config.color,
            thickness: config.thickness,
            thickness_type: config.thickness_type,
            alignment: config.alignment,
            hollow: config.hollow,
            vertices: [vertex0, vertex1, vertex2],
            roundness: config.roundness,
        }
    }
}

impl ShapeComponent for Triangle {
    type Data = TriangleData;

    fn into_data(&self, tf: &GlobalTransform) -> TriangleData {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);

        TriangleData {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_linear_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            vertex_0: self.vertices[0].into(),
            vertex_1: self.vertices[1].into(),
            vertex_2: self.vertices[2].into(),
            roundness: self.roundness,
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
#[derive(Clone, Copy, Reflect, Pod, Zeroable, Default, Debug)]
#[repr(C)]
pub struct TriangleData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    vertex_0: [f32; 2],
    vertex_1: [f32; 2],
    vertex_2: [f32; 2],
    roundness: f32,
}

impl TriangleData {
    pub fn new(config: &ShapeConfig, vertex_0: Vec2, vertex_1: Vec2, vertex_2: Vec2) -> TriangleData {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);

        TriangleData {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,
            vertex_0: vertex_0.into(),
            vertex_1: vertex_1.into(),
            vertex_2: vertex_2.into(),
            roundness: config.roundness,
        }
    }
}

impl ShapeData for TriangleData {
    type Component = Triangle;

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
        TRIANGLE_HANDLE.typed::<Shader>().into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw triangles.
pub trait TrianglePainter {
    fn triangle(&mut self, vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> &mut Self;
}

impl<'w, 's> TrianglePainter for ShapePainter<'w, 's> {
    fn triangle(&mut self, vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> &mut Self {
        self.send(TriangleData::new(self.config(), vertex0, vertex1, vertex2))
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of triangle bundles.
pub trait TriangleBundle {
    fn triangle(config: &ShapeConfig, vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> Self;
}

impl TriangleBundle for ShapeBundle<Triangle> {
    fn triangle(config: &ShapeConfig, vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> Self {
        Self::new(config, Triangle::new(config, vertex0, vertex1, vertex2))
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of triangle entities.
pub trait TriangleSpawner<'w, 's> {
    fn triangle(
        &mut self,
        vertex0: Vec2,
        vertex1: Vec2,
        vertex2: Vec2,
    ) -> ShapeEntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: ShapeSpawner<'w, 's>> TriangleSpawner<'w, 's> for T {
    fn triangle(
        &mut self,
        vertex0: Vec2,
        vertex1: Vec2,
        vertex2: Vec2,
    ) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::triangle(
            self.config(),
            vertex0,
            vertex1,
            vertex2,
        ))
    }
}
