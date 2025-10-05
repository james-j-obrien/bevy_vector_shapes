use bevy::{prelude::*, reflect::Reflect, render::render_resource::ShaderType, shader::ShaderRef};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, DISC_HANDLE},
};

/// Component containing the data for drawing a disc.
///
/// Discs include both arcs and circles
#[derive(Component, Reflect)]
pub struct DiscComponent {
    pub alignment: Alignment,
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

impl DiscComponent {
    pub fn new(
        config: &ShapeConfig,
        radius: f32,
        arc: bool,
        start_angle: f32,
        end_angle: f32,
        cap: Cap,
    ) -> Self {
        Self {
            alignment: config.alignment,
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

impl ShapeComponent for DiscComponent {
    type Data = DiscData;

    fn get_data(&self, tf: &GlobalTransform, fill: &ShapeFill) -> DiscData {
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
        flags.set_arc(self.arc as u32);

        DiscData {
            transform: tf.to_matrix().to_cols_array_2d(),

            color: fill.color.to_linear().to_f32_array(),
            thickness,
            flags: flags.0,

            radius: self.radius,
            start_angle: self.start_angle,
            end_angle: self.end_angle,

            padding: default(),
        }
    }
}

impl Default for DiscComponent {
    fn default() -> Self {
        Self {
            alignment: default(),
            cap: Cap::None,
            arc: false,

            radius: 1.0,
            start_angle: 0.0,
            end_angle: 0.0,
        }
    }
}

/// Raw data sent to the disc shader to draw a disc
#[derive(Clone, Copy, Reflect, Default, Debug, ShaderType)]
#[repr(C)]
pub struct DiscData {
    transform: [[f32; 4]; 4],

    color: [f32; 4],
    thickness: f32,
    flags: u32,

    radius: f32,
    start_angle: f32,
    end_angle: f32,

    padding: [f32; 3],
}

impl DiscData {
    pub fn circle(config: &ShapeConfig, radius: f32) -> DiscData {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);
        flags.set_arc(false as u32);

        DiscData {
            transform: config.transform.to_matrix().to_cols_array_2d(),

            color: config.color.to_linear().to_f32_array(),
            thickness: config.thickness,
            flags: flags.0,

            radius,

            start_angle: 0.0,
            end_angle: 0.0,

            padding: default(),
        }
    }

    pub fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> DiscData {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);
        flags.set_cap(config.cap);
        flags.set_arc(true as u32);

        DiscData {
            transform: config.transform.to_matrix().to_cols_array_2d(),

            color: config.color.to_linear().to_f32_array(),
            thickness: config.thickness,
            flags: flags.0,

            radius,

            start_angle,
            end_angle,

            padding: default(),
        }
    }
}

impl ShapeData for DiscData {
    type Component = DiscComponent;

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

    fn shader() -> ShaderRef {
        DISC_HANDLE.into()
    }

    fn transform(&self) -> Mat4 {
        Mat4::from_cols_array_2d(&self.transform)
    }
}

/// Extension trait for [`ShapePainter`] to enable it to draw disc type shapes.
pub trait DiscPainter {
    fn circle(&mut self, radius: f32) -> &mut Self;
    fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> &mut Self;
}

impl<'w, 's> DiscPainter for ShapePainter<'w, 's> {
    fn circle(&mut self, radius: f32) -> &mut Self {
        self.send(DiscData::circle(self.config(), radius))
    }

    fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> &mut Self {
        self.send(DiscData::arc(self.config(), radius, start_angle, end_angle));
        self
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of bundles for disc type shapes.
pub trait DiscBundle {
    fn circle(config: &ShapeConfig, radius: f32) -> Self;
    fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> Self;
}

impl DiscBundle for ShapeBundle<DiscComponent> {
    fn circle(config: &ShapeConfig, radius: f32) -> Self {
        Self::new(config, DiscComponent::circle(config, radius))
    }

    fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        Self::new(
            config,
            DiscComponent::arc(config, radius, start_angle, end_angle),
        )
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of entities for disc type shapes.
pub trait DiscSpawner<'w> {
    fn circle(&mut self, radius: f32) -> ShapeEntityCommands;
    fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> ShapeEntityCommands;
}

impl<'w, T: ShapeSpawner<'w>> DiscSpawner<'w> for T {
    fn circle(&mut self, radius: f32) -> ShapeEntityCommands {
        self.spawn_shape(ShapeBundle::circle(self.config(), radius))
    }

    fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> ShapeEntityCommands {
        self.spawn_shape(ShapeBundle::arc(
            self.config(),
            radius,
            start_angle,
            end_angle,
        ))
    }
}
