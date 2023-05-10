use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::{prelude::*, ShapeMode};

/// Describes a configuration that can be applied to a spawned shape.
#[derive(Copy, Clone, Reflect, FromReflect)]
pub struct ShapeConfig {
    /// Transform with which the shape will be spawned
    pub transform: Transform,

    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    /// If true spawned shape will be hollow, taking into account thickness and thickness_type
    pub hollow: bool,
    pub cap: Cap,
    pub roundness: f32,
    pub corner_radii: Vec4,

    #[reflect(ignore)]
    pub render_layers: Option<RenderLayers>,
    pub alpha_mode: AlphaMode,
    /// Forcibly disables local anti-aliasing for all shapes
    pub disable_laa: bool,
    pub canvas: Option<Entity>,
    pub mode: ShapeMode,
}

impl ShapeConfig {
    /// Helper method to modify the configs transform taking into account rotation and scale.
    pub fn translate(&mut self, dir: Vec3) {
        self.transform.translation += self.transform.rotation * dir * self.transform.scale;
    }

    /// Helper method to set the configs transform.
    pub fn set_translation(&mut self, translation: Vec3) {
        self.transform.translation = translation;
    }

    /// Helper method to rotate the configs transform by a given [`Quat`].
    pub fn rotate(&mut self, quat: Quat) {
        self.transform.rotation *= quat;
    }

    /// Helper method to set the configs rotation.
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.transform.rotation = rotation;
    }

    /// Helper method to rotate the configs transform around the x axis.
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_x(angle))
    }

    /// Helper method to rotate the configs transform around the y axis.
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_y(angle))
    }

    /// Helper method to rotate the configs transform around the z axis.
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_z(angle))
    }

    /// Helper method to scale the configs transform.
    pub fn scale(&mut self, scale: Vec3) {
        self.transform.scale *= scale;
    }

    /// Helper method to set the configs scale.
    pub fn set_scale(&mut self, scale: Vec3) {
        self.transform.scale = scale;
    }

    pub fn without_transform(&self) -> Self {
        let mut config = self.clone();
        config.transform = Transform::IDENTITY;
        config
    }
}

impl ShapeConfig {
    pub fn default_2d() -> Self {
        Self {
            transform: default(),

            color: Color::GRAY,
            thickness: 0.1,
            thickness_type: default(),
            alignment: default(),
            hollow: false,
            cap: default(),
            roundness: default(),
            corner_radii: default(),

            render_layers: None,
            alpha_mode: AlphaMode::Blend,
            disable_laa: false,
            canvas: None,
            mode: ShapeMode::Shape2d,
        }
    }
}

impl ShapeConfig {
    pub fn default_3d() -> Self {
        let mut config = Self::default_2d();
        config.mode = ShapeMode::Shape3d;
        config
    }
}
