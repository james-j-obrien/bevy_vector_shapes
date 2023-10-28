use bevy::ecs::component::Tick;
use bevy::ecs::system::{SystemMeta, SystemParam};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::utils::synccell::SyncCell;

use crate::prelude::*;
use crate::render::ShapePipelineType;

/// Describes a configuration that can be applied to a spawned shape.
#[derive(Clone, Reflect)]
pub struct ShapeConfig {
    /// Transform with which the shape will be spawned.
    pub transform: Transform,

    pub color: Color,
    pub thickness: f32,
    pub thickness_type: ThicknessType,
    pub alignment: Alignment,
    /// If true spawned shape will be hollow, taking into account thickness and thickness_type.
    pub hollow: bool,
    pub cap: Cap,
    pub roundness: f32,
    pub corner_radii: Vec4,

    #[reflect(ignore)]
    pub render_layers: Option<RenderLayers>,
    pub alpha_mode: AlphaMode,
    /// Forcibly disables local anti-aliasing.
    pub disable_laa: bool,
    /// [`Canvas`] to draw the shape to.
    pub canvas: Option<Entity>,
    /// Texture to apply to the shape, color is determined as color * sample.
    pub texture: Option<Handle<Image>>,
    /// Set with set_2d, set_3d and set_canvas.
    pub pipeline: ShapePipelineType,
    /// Indicates whether or not the config will be reset after a system is run
    pub reset: bool,
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

    /// Helper method to change shape render target to a canvas.
    ///
    /// Also sets pipeline to Shape2d.
    pub fn set_canvas(&mut self, canvas: Entity) {
        self.pipeline = ShapePipelineType::Shape2d;
        self.canvas = Some(canvas);
    }

    /// Helper method to change the target pipeline to the 3d pipeline.
    pub fn set_3d(&mut self) {
        self.pipeline = ShapePipelineType::Shape3d;
    }

    /// Helper method to change the target pipeline to the 2d pipeline.
    pub fn set_2d(&mut self) {
        self.pipeline = ShapePipelineType::Shape2d;
    }

    /// Helper method to clone the config without it's transform, useful when parenting.
    pub fn without_transform(&self) -> Self {
        let mut config = self.clone();
        config.transform = Transform::IDENTITY;
        config
    }
}

impl ShapeConfig {
    /// Default [`ShapeConfig`] with target set to the 2D pipeline.
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
            texture: None,
            pipeline: ShapePipelineType::Shape2d,
            reset: true,
        }
    }
}

impl ShapeConfig {
    /// Default [`ShapeConfig`] with target set to the 3D pipeline.
    pub fn default_3d() -> Self {
        let mut config = Self::default_2d();
        config.pipeline = ShapePipelineType::Shape3d;
        config
    }
}

impl FromWorld for ShapeConfig {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<BaseShapeConfig>();
        config.0.clone()
    }
}

unsafe impl<'r> SystemParam for &'r mut ShapeConfig {
    type State = SyncCell<ShapeConfig>;
    type Item<'w, 's> = &'s mut ShapeConfig;

    fn init_state(world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        SyncCell::new(ShapeConfig::from_world(world))
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
        _world: UnsafeWorldCell<'w>,
        _change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        state.get()
    }

    fn apply(state: &mut Self::State, _system_meta: &SystemMeta, world: &mut World) {
        let state = state.get();
        if state.reset {
            *state = world.resource::<BaseShapeConfig>().0.clone();
        }
    }
}
