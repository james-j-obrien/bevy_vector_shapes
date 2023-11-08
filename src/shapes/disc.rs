use bevy::{
    prelude::*,
    reflect::Reflect,
    render::render_resource::{ShaderRef, ShaderType},
};
use wgpu::vertex_attr_array;

use crate::{
    prelude::*,
    render::{Flags, ShapeComponent, ShapeData, DISC_HANDLE},
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
    /// width and height of the disc, overwrites radius if set
    pub extents: Option<Vec2>,
    /// Starting angle for an arc
    pub start_angle: f32,
    /// Ending angle for an arc
    pub end_angle: f32,
}

impl Disc {
    pub fn new(
        config: &ShapeConfig,
        radius: f32,
        extents: Option<Vec2>,
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
            extents,
            start_angle,
            end_angle,
        }
    }

    pub fn circle(config: &ShapeConfig, radius: f32) -> Self {
        Self::new(config, radius, None, false, 0.0, 0.0, Cap::None)
    }

    pub fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        Self::new(
            config,
            radius,
            None,
            true,
            start_angle,
            end_angle,
            config.cap,
        )
    }

    pub fn ellipse(config: &ShapeConfig, extents: Vec2) -> Self {
        Self::new(config, 0.0, Some(extents), false, 0.0, 0.0, Cap::None)
    }
}

impl ShapeComponent for Disc {
    type Data = DiscData;

    fn get_data(&self, tf: &GlobalTransform) -> DiscData {
        let mut flags = Flags(0);
        flags.set_thickness_type(self.thickness_type);
        flags.set_alignment(self.alignment);
        flags.set_hollow(self.hollow as u32);
        flags.set_cap(self.cap);
        flags.set_arc(self.arc as u32);

        // adjust transfom if extents are set
        let (radius, transform) = match self.extents {
            Some(extents) => {
                let mut transform = tf.compute_transform(); // annoying, but makes it so much easier

                let radius = (extents.x + extents.y) / 2.0; // base radius, will be distorted by scale to achieve ellipse
                transform.scale.x *= extents.x / radius;
                transform.scale.y *= extents.y / radius;
                (radius, transform)
            }
            None => (self.radius, tf.compute_transform()), // compute transform here too for matching types
        };

        DiscData {
            transform: transform.compute_matrix().to_cols_array_2d(),

            color: self.color.as_linear_rgba_f32(),
            thickness: self.thickness,
            flags: flags.0,

            radius: radius,
            start_angle: self.start_angle,
            end_angle: self.end_angle,

            padding: default(),
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
            extents: None,
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
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            radius,

            start_angle: 0.0,
            end_angle: 0.0,

            padding: default(),
        }
    }

    pub fn ellipse(config: &ShapeConfig, extents: Vec2) -> DiscData {
        let mut flags = Flags(0);
        flags.set_thickness_type(config.thickness_type);
        flags.set_alignment(config.alignment);
        flags.set_hollow(config.hollow as u32);
        flags.set_arc(false as u32);

        let mut transform = config.transform;

        let radius = (extents.x + extents.y) / 2.0; // base radius, will be distorted by scale to achieve ellipse
        transform.scale.x *= extents.x / radius;
        transform.scale.y *= extents.y / radius;

        DiscData {
            transform: transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
            thickness: config.thickness,
            flags: flags.0,

            radius,

            start_angle: 0.0,
            end_angle: 0.0,
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
            transform: config.transform.compute_matrix().to_cols_array_2d(),

            color: config.color.as_linear_rgba_f32(),
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
    fn ellipse(&mut self, extents: Vec2) -> &mut Self;
}

impl<'w, 's> DiscPainter for ShapePainter<'w, 's> {
    fn circle(&mut self, radius: f32) -> &mut Self {
        self.send(DiscData::circle(self.config(), radius))
    }

    fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> &mut Self {
        self.send(DiscData::arc(self.config(), radius, start_angle, end_angle));
        self
    }

    fn ellipse(&mut self, extents: Vec2) -> &mut Self {
        self.send(DiscData::ellipse(self.config(), extents))
    }
}

/// Extension trait for [`ShapeBundle`] to enable creation of bundles for disc type shapes.
pub trait DiscBundle {
    fn circle(config: &ShapeConfig, radius: f32) -> Self;
    fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> Self;
    fn ellipse(config: &ShapeConfig, extents: Vec2) -> Self;
}

impl DiscBundle for ShapeBundle<Disc> {
    fn circle(config: &ShapeConfig, radius: f32) -> Self {
        Self::new(config, Disc::circle(config, radius))
    }

    fn arc(config: &ShapeConfig, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        Self::new(config, Disc::arc(config, radius, start_angle, end_angle))
    }

    fn ellipse(config: &ShapeConfig, extents: Vec2) -> Self {
        Self::new(config, Disc::ellipse(config, extents))
    }
}

/// Extension trait for [`ShapeSpawner`] to enable spawning of entities for disc type shapes.
pub trait DiscSpawner<'w, 's> {
    fn circle(&mut self, radius: f32) -> ShapeEntityCommands<'w, 's, '_>;
    fn arc(
        &mut self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> ShapeEntityCommands<'w, 's, '_>;
    fn ellipse(&mut self, extents: Vec2) -> ShapeEntityCommands<'w, 's, '_>;
}

impl<'w, 's, T: ShapeSpawner<'w, 's>> DiscSpawner<'w, 's> for T {
    fn circle(&mut self, radius: f32) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::circle(self.config(), radius))
    }

    fn arc(
        &mut self,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::arc(
            self.config(),
            radius,
            start_angle,
            end_angle,
        ))
    }
    fn ellipse(&mut self, extents: Vec2) -> ShapeEntityCommands<'w, 's, '_> {
        self.spawn_shape(ShapeBundle::ellipse(self.config(), extents))
    }
}
