use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    reflect::{FromReflect, Reflect},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{
        setup_instanced_pipeline, setup_instanced_pipeline_2d, Flags, InstanceComponent,
        Instanceable, NGON_HANDLE,
    },
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

impl InstanceComponent<NgonInstance> for RegularPolygon {
    fn instance(&self, tf: &GlobalTransform) -> NgonInstance {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);

        NgonInstance {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            sides: self.sides,
            radius: self.radius,
            roundness: self.roundness,
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
#[derive(Clone, Copy, Reflect, FromReflect, Pod, Zeroable)]
#[repr(C)]
pub struct NgonInstance {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    sides: f32,
    radius: f32,
    roundness: f32,
}

impl Instanceable for NgonInstance {
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

    fn shader() -> Handle<Shader> {
        NGON_HANDLE.typed::<Shader>()
    }

    fn distance(&self) -> f32 {
        self.transform().transform_point3(Vec3::ZERO).z
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

pub(crate) struct RegularPolygonPlugin;

impl Plugin for RegularPolygonPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RegularPolygon>();
        setup_instanced_pipeline::<NgonInstance>(app);
    }
}

pub(crate) struct RegularPolygon2dPlugin;

impl Plugin for RegularPolygon2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RegularPolygon>();
        setup_instanced_pipeline_2d::<NgonInstance>(app);
    }
}
