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
        Instanceable, LINE_HANDLE,
    },
};

/// Component containing the data for drawing a line.
#[derive(Component, Reflect)]
pub struct Line {
    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    pub cap: Cap,

    /// Position to draw the start of the line in world space relative to it's transform.
    pub start: Vec3,
    /// Position to draw the end of the line in world space relative to it's transform.
    pub end: Vec3,
}

impl Line {
    pub fn new(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        Self {
            color: config.color,
            thickness: config.thickness,
            thickness_type: config.thickness_type,
            alignment: config.alignment,
            cap: config.cap,

            start,
            end,
        }
    }
}

impl Default for Line {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            thickness: 1.0,
            thickness_type: default(),
            alignment: default(),
            cap: default(),

            start: default(),
            end: default(),
        }
    }
}

impl InstanceComponent<LineInstance> for Line {
    fn instance(&self, tf: &GlobalTransform) -> LineInstance {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_cap(self.cap);

        LineInstance {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            start: self.start,
            end: self.end,
        }
    }
}

/// Raw data sent to the line shader to draw a line
#[derive(Clone, Copy, Reflect, FromReflect, Pod, Zeroable)]
#[repr(C)]
pub struct LineInstance {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    start: Vec3,
    end: Vec3,
}

impl LineInstance {
    pub fn new(config: &ShapeConfig, start: Vec3, end: Vec3) -> Self {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_cap(config.cap);

        LineInstance {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            start,
            end,
        }
    }
}

impl Instanceable for LineInstance {
    type Component = Line;

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

    fn shader() -> Handle<Shader> {
        LINE_HANDLE.typed::<Shader>()
    }

    fn distance(&self) -> f32 {
        self.transform().transform_point3(Vec3::ZERO).z
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

pub(crate) struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Line>();
        setup_instanced_pipeline::<LineInstance>(app)
    }
}

pub(crate) struct Line2dPlugin;

impl Plugin for Line2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Line>();
        setup_instanced_pipeline_2d::<LineInstance>(app)
    }
}
