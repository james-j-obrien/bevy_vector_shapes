use bevy::{
    prelude::*,
    reflect::{Reflect},
    render::render_resource::ShaderType,
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{
        setup_instanced_pipeline, setup_instanced_pipeline_2d, Flags, InstanceComponent,
        Instanceable, RECT_HANDLE,
    },
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

impl InstanceComponent<RectInstance> for Rectangle {
    fn instance(&self, tf: &GlobalTransform) -> RectInstance {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);

        RectInstance {
            transform: tf.compute_matrix().to_cols_array_2d(),

            color: self.color.as_rgba_f32(),
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
#[derive(Component, ShaderType, Clone, Copy)]
#[repr(C, align(16))]
pub struct RectInstance {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    size: [f32; 2],
    corner_radii: [f32; 4],
}

impl RectInstance {
    pub fn new(config: &ShapeConfig, size: Vec2) -> Self {
        let mut flags = Flags(0);
        flags.set_alignment(config.alignment);
        flags.set_thickness_type(config.thickness_type);
        flags.set_hollow(config.hollow as u32);

        Self {
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            size: size.into(),
            corner_radii: config.corner_radii.into(),
        }
    }
}

impl Instanceable for RectInstance {
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

    fn shader() -> Handle<Shader> {
        RECT_HANDLE.typed::<Shader>()
    }

    fn distance(&self) -> f32 {
        self.transform().transform_point3(Vec3::ZERO).z
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }

    fn null_instance() -> Self {
        let config = ShapeConfig::default();
        Self::new(&config, Vec2::ZERO)
    }
}

pub(crate) struct RectPlugin;

impl Plugin for RectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Rectangle>();
        setup_instanced_pipeline::<RectInstance>(app);
    }
}

pub(crate) struct Rect2dPlugin;

impl Plugin for Rect2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Rectangle>();
        setup_instanced_pipeline_2d::<RectInstance>(app);
    }
}
