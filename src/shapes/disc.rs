use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    reflect::Reflect,
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{
        setup_instanced_pipeline, setup_instanced_pipeline_2d, Flags, InstanceComponent,
        Instanceable, DISC_HANDLE,
    },
};

/// Component containing the data for drawing a disc.
///
/// Discs include both arcs and circles
#[derive(Component, Reflect)]
pub struct Disc {
    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    pub hollow: bool,
    /// Cap type for an arc, only supports None or Round
    pub cap: Cap,
    /// Whether to treat this disc like an arc
    pub arc: bool,

    /// External radius of the disc
    pub radius: f32,
    /// Starting angle for an arc
    pub start_angle: f32,
    /// Ending angle for an arc
    pub end_angle: f32,
}

impl Disc {
    pub fn new(
        config: &ShapeConfig,
        radius: f32,
        arc: bool,
        start_angle: f32,
        end_angle: f32,
        cap: Cap,
    ) -> Self {
        Self {
            color: config.color,
            thickness: config.thickness,
            thickness_type: config.thickness_type,
            alignment: config.alignment,
            hollow: config.hollow,
            cap,
            arc,

            radius,
            start_angle,
            end_angle,
        }
    }

    pub fn circle(config: &ShapeConfig, radius: f32) -> Self {
        Self::new(config, radius, false, 0.0, 0.0, Cap::None)
    }

    pub fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        Self::new(config, radius, true, start_angle, end_angle, config.cap)
    }
}

impl InstanceComponent<DiscInstance> for Disc {
    fn instance(&self, tf: &GlobalTransform) -> DiscInstance {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);
        flags.set_cap(self.cap);
        flags.set_arc(self.arc as u32);

        DiscInstance {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            radius: self.radius,
            start_angle: self.start_angle,
            end_angle: self.end_angle,
        }
    }
}

impl Default for Disc {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            thickness: 1.0,
            thickness_type: default(),
            alignment: default(),
            hollow: false,
            cap: Cap::None,
            arc: false,

            radius: 1.0,
            start_angle: 0.0,
            end_angle: 0.0,
        }
    }
}

/// Raw data sent to the disc shader to draw a disc
#[derive(Clone, Copy, Reflect, FromReflect, Pod, Zeroable)]
#[repr(C)]
pub struct DiscInstance {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    radius: f32,
    start_angle: f32,
    end_angle: f32,
}

impl DiscInstance {
    pub fn circle(config: &ShapeConfig, radius: f32) -> DiscInstance {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);
        flags.set_cap(config.cap);
        flags.set_arc(false as u32);

        DiscInstance {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            radius,

            start_angle: 0.0,
            end_angle: 0.0,
        }
    }

    pub fn arc(
        config: &ShapeConfig,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> DiscInstance {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);
        flags.set_cap(config.cap);
        flags.set_arc(true as u32);

        DiscInstance {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            radius,

            start_angle,
            end_angle,
        }
    }
}

impl Instanceable for DiscInstance {
    type Component = Disc;

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
            9 => Float32,
        ]
        .to_vec()
    }

    fn shader() -> Handle<Shader> {
        DISC_HANDLE.typed::<Shader>()
    }

    fn distance(&self) -> f32 {
        self.transform().transform_point3(Vec3::ZERO).z
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

pub(crate) struct DiscPlugin;

impl Plugin for DiscPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Disc>();
        setup_instanced_pipeline::<DiscInstance>(app);
    }
}

pub(crate) struct Disc2dPlugin;

impl Plugin for Disc2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Disc>();
        setup_instanced_pipeline_2d::<DiscInstance>(app);
    }
}
