use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    slice::Iter,
};

use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};

use any_vec::AnyVec;

use crate::{
    prelude::*,
    render::{ShapeData, ShapeInstance, ShapePipelineMaterial, ShapePipelineType},
};

/// A system param for type erased storage of [`ShapeInstance`].
///
/// Generally should only be consumed as part of [`ShapePainter`] and not used directly.
#[derive(Resource, Default)]
pub struct ShapeStorage {
    shapes: HashMap<(TypeId, ShapePipelineType), AnyVec<dyn Send + Sync>>,
}

impl ShapeStorage {
    fn send<T: ShapeData>(&mut self, config: &ShapeConfig, data: T) {
        let key = (TypeId::of::<T>(), config.pipeline);
        let entry = (ShapePipelineMaterial::from(config), data);
        let vec = self
            .shapes
            .entry(key)
            .or_insert_with(AnyVec::new::<ShapeInstance<T>>);

        // SAFETY: we only insert entries in this function and only those that match the appropriate TypeId
        unsafe {
            vec.downcast_mut_unchecked().push(entry);
        }
    }

    pub fn get<T: ShapeData>(
        &self,
        pipeline: ShapePipelineType,
    ) -> Option<Iter<'_, ShapeInstance<T>>> {
        // SAFETY: we only insert entries in ShapeStorage::send and only those that match the appropriate TypeId
        self.shapes
            .get(&(TypeId::of::<T>(), pipeline))
            .map(|vec| unsafe { vec.downcast_ref_unchecked::<ShapeInstance<T>>().iter() })
    }

    fn clear(&mut self) {
        self.shapes = HashMap::new();
    }
}

/// Clears the [`ShapeStorage`] resource each frame.
pub fn clear_storage(mut storage: ResMut<ShapeStorage>) {
    storage.clear();
}

/// A system param that allows ergonomic drawing of immediate mode shapes.
///
/// The [`ShapeConfig`] used is initially extracted from the [`BaseShapeConfig`] resource.
/// Subsequent calls to `reset()` will reset the config back to whatever is currently stored within the [`BaseShapeConfig`] resource.
///
/// Shapes are spawned via events which will be extracted for rendering.
#[derive(SystemParam)]
pub struct ShapePainter<'w, 's> {
    config: &'s mut ShapeConfig,
    shapes: ResMut<'w, ShapeStorage>,
    default_config: Res<'w, BaseShapeConfig>,
}

impl<'w, 's> ShapePainter<'w, 's> {
    pub fn config(&self) -> &ShapeConfig {
        self.config
    }

    pub fn set_config(&mut self, config: ShapeConfig) {
        *self.config = config;
    }

    pub fn send<T: ShapeData>(&mut self, data: T) -> &mut Self {
        let Self {
            config,
            shapes: event_writer,
            ..
        } = self;
        event_writer.send(config, data);
        self
    }

    pub fn send_with_config<T: ShapeData>(&mut self, config: &ShapeConfig, data: T) -> &mut Self {
        self.shapes.send(config, data);
        self
    }

    /// Takes a closure which builds children for this shape.
    ///
    /// While event based shapes don't have the parent child relationship that entities have,
    /// this API allows parity between the behaviour of [`ShapeCommands`] and [`ShapePainter`]
    pub fn with_children(&mut self, spawn_children: impl FnOnce(&mut ShapePainter)) -> &mut Self {
        let config = self.config.clone();
        spawn_children(self);
        *self.config = config;
        self
    }

    /// Set the painter's [`ShapeConfig`] to the current value of the [`BaseShapeConfig`] resource.
    pub fn reset(&mut self) {
        *self.config = self.default_config.0.clone();
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
