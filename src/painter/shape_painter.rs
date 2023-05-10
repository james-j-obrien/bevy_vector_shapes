use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    slice::Iter,
};

use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};

use any_vec::AnyVec;

use crate::{
    painter::LocalShapeConfig,
    prelude::*,
    render::{RenderKey, ShapeData},
    ShapeMode,
};

/// Type stored in ShapeStorage
pub type ShapeEntry<T> = (RenderKey, T);

/// A system param for type erased storage of [`ShapeEntry`]
///
/// Generally should only be consumed as part of [`ShapePainter`] and not used directly.
#[derive(Resource, Default)]
pub struct ShapeStorage {
    shapes: HashMap<(TypeId, ShapeMode), AnyVec<dyn Send + Sync>>,
}

impl ShapeStorage {
    fn send<T: ShapeData>(&mut self, config: &ShapeConfig, instance: T) {
        let key = (TypeId::of::<T>(), config.mode);
        let entry = (RenderKey::from(config), instance);
        let vec = self
            .shapes
            .entry(key)
            .or_insert_with(|| AnyVec::new::<ShapeEntry<T>>());

        // SAFETY: we only insert entries in this function and only those that match the appropriate TypeId
        unsafe {
            vec.downcast_mut_unchecked().push(entry);
        }
    }

    pub fn get<T: ShapeData>(&self, mode: ShapeMode) -> Option<Iter<'_, ShapeEntry<T>>> {
        match self.shapes.get(&(TypeId::of::<T>(), mode)) {
            // SAFETY: we only insert entries in ShapeStorage::send and only those that match the appropriate TypeId
            Some(vec) => Some(unsafe { vec.downcast_ref_unchecked::<ShapeEntry<T>>().iter() }),
            None => None,
        }
    }

    fn clear(&mut self) {
        self.shapes = HashMap::new();
    }
}

pub fn clear_storage(mut storage: ResMut<ShapeStorage>) {
    storage.clear();
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
    event_writer: ResMut<'w, ShapeStorage>,
    default_config: Res<'w, BaseShapeConfig>,
}

impl<'w, 's> ShapePainter<'w, 's> {
    pub fn config(&self) -> &ShapeConfig {
        &self.config.0
    }

    pub fn set_config(&mut self, config: &ShapeConfig) {
        self.config.0 = *config;
    }

    pub fn send<T: ShapeData>(&mut self, instance: T) -> &mut Self {
        let Self {
            config,
            event_writer,
            ..
        } = self;
        event_writer.send(config, instance);
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
    pub fn reset(&mut self) {
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
