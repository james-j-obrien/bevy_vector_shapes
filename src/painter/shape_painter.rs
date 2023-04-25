use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    painter::LocalShapeConfig,
    prelude::*,
    render::{Instanceable, RenderKey},
    shapes::{DiscInstance, LineInstance, NgonInstance, RectInstance},
};

/// Event backing immediate mode shapes.
pub struct ShapeEvent<T: Instanceable>(pub(crate) (RenderKey, T));

/// A system param for writing each type of [`ShapeEvent`]
///
/// Generally should only be consumed as part of [`ShapePainter`] and not used directly.
#[derive(SystemParam)]
pub struct ShapeEventWriter<'w> {
    line_writer: EventWriter<'w, ShapeEvent<LineInstance>>,
    rect_writer: EventWriter<'w, ShapeEvent<RectInstance>>,
    disc_writer: EventWriter<'w, ShapeEvent<DiscInstance>>,
    ngon_writer: EventWriter<'w, ShapeEvent<NgonInstance>>,
}

impl<'w> ShapeEventWriter<'w> {
    fn line(&mut self, render_key: RenderKey, instance: LineInstance) {
        self.line_writer.send(ShapeEvent((render_key, instance)));
    }

    fn rect(&mut self, render_key: RenderKey, instance: RectInstance) {
        self.rect_writer.send(ShapeEvent((render_key, instance)));
    }

    fn disc(&mut self, render_key: RenderKey, instance: DiscInstance) {
        self.disc_writer.send(ShapeEvent((render_key, instance)));
    }

    fn ngon(&mut self, render_key: RenderKey, instance: NgonInstance) {
        self.ngon_writer.send(ShapeEvent((render_key, instance)));
    }
}

/// A system param that allows ergonomic drawing of immediate mode shapes.
///
/// The ShapeConfig used is initially extracted from the [`BaseShapeConfig`] resource.
/// Subsequent calls to .clear() will reset the config back to whatever is currently stored within the [`BaseShapeConfig`] resource.
///
/// Shapes are spawned via events which will be extracted for rendering.
#[derive(SystemParam)]
pub struct ShapePainter<'w, 's> {
    config: Local<'s, LocalShapeConfig>,
    event_writer: ShapeEventWriter<'w>,
    default_config: Res<'w, BaseShapeConfig>,
}

impl<'w, 's> ShapePainter<'w, 's> {
    pub fn config(&self) -> &ShapeConfig {
        &self.config.0
    }

    pub fn set_config(&mut self, config: &ShapeConfig) {
        self.config.0 = *config;
    }

    pub fn line(&mut self, start: Vec3, end: Vec3) -> &mut Self {
        self.event_writer.line(
            RenderKey::from(self.config()),
            LineInstance::new(self.config(), start, end),
        );
        self
    }

    pub fn rect(&mut self, size: Vec2) -> &mut Self {
        self.event_writer.rect(
            RenderKey::from(self.config()),
            RectInstance::new(self.config(), size),
        );
        self
    }

    pub fn ngon(&mut self, sides: f32, radius: f32) -> &mut Self {
        self.event_writer.ngon(
            RenderKey::from(self.config()),
            NgonInstance::new(self.config(), sides, radius),
        );
        self
    }

    pub fn circle(&mut self, radius: f32) -> &mut Self {
        self.event_writer.disc(
            RenderKey::from(self.config()),
            DiscInstance::circle(self.config(), radius),
        );
        self
    }

    pub fn arc(&mut self, radius: f32, start_angle: f32, end_angle: f32) -> &mut Self {
        self.event_writer.disc(
            RenderKey::from(self.config()),
            DiscInstance::arc(self.config(), radius, start_angle, end_angle),
        );
        self
    }

    /// Takes a closure which builds children for this shape.
    ///
    /// While event based shapes don't have the parent child relationship that entities have,
    /// this API allows parity between the behaviour of [`ShapeCommands`] and [`ShapePainter`]
    pub fn with_children(&mut self, spawn_children: impl FnOnce(&mut ShapePainter)) -> &mut Self {
        let config = self.config.clone();
        spawn_children(self);
        self.config.0 = config;
        self
    }

    /// Set the painter's [`ShapeConfig`] to the current value of the [`BaseShapeConfig`] resource.
    pub fn clear(&mut self) {
        self.config.0 = self.default_config.0;
    }
}

impl<'w, 's> Deref for ShapePainter<'w, 's> {
    type Target = ShapeConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<'w, 's> DerefMut for ShapePainter<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}
