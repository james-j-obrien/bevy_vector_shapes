use std::ops::{Deref, DerefMut};

use bevy::{
    ecs::{system::EntityCommands, world::Command},
    prelude::*,
};
use smallvec::SmallVec;

use crate::{prelude::*, render::ShapePipelineType};

/// Command that pushes children to the end of the entity's [`Children`].
///
/// Duplicated here from [`bevy::prelude::PushChildren`] in order to access private internals.
#[derive(Debug)]
pub struct PushChildren {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
}

impl Command for PushChildren {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.parent).push_children(&self.children);
    }
}

/// [`EntityCommands`] that also stores [`ShapeConfig`] for easier spawning of child shapes.
pub struct ShapeEntityCommands<'w, 's> {
    pub commands: EntityCommands<'w>,
    pub config: &'s ShapeConfig,
}

impl<'w, 's> ShapeEntityCommands<'w, 's> {
    /// Takes a closure which builds children for this entity using [`ShapeChildBuilder`].
    pub fn with_children(
        &mut self,
        spawn_children: impl FnOnce(&mut ShapeChildBuilder),
    ) -> &mut Self {
        let config = self.config.without_transform();
        let parent = self.id();
        let mut painter = ShapeChildBuilder {
            commands: self.commands(),
            push_children: PushChildren {
                children: SmallVec::default(),
                parent,
            },
            config,
        };

        spawn_children(&mut painter);
        let children = painter.push_children;
        self.commands().add(children);
        self
    }
}

impl<'w, 's> Deref for ShapeEntityCommands<'w, 's> {
    type Target = EntityCommands<'w>;

    fn deref(&self) -> &Self::Target {
        &self.commands
    }
}

impl<'w, 's> DerefMut for ShapeEntityCommands<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.commands
    }
}

/// [`ChildBuilder`] that also exposes shape spawning methods from [`ShapeCommands`].
pub struct ShapeChildBuilder<'w> {
    commands: Commands<'w, 'w>,
    config: ShapeConfig,
    push_children: PushChildren,
}

impl<'w> ShapeChildBuilder<'w> {
    /// Spawns an entity with the given bundle and inserts it into the parent entity's [`Children`].
    /// Also adds [`Parent`] component to the created entity.
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands {
        let e = self.commands.spawn(bundle);
        self.push_children.children.push(e.id());
        e
    }

    /// Spawns an [`Entity`] with no components and inserts it into the parent entity's [`Children`].
    /// Also adds [`Parent`] component to the created entity.
    pub fn spawn_empty(&mut self) -> EntityCommands {
        let e = self.commands.spawn_empty();
        self.push_children.children.push(e.id());
        e
    }

    /// Returns the parent entity of this [`ChildBuilder`].
    pub fn parent_entity(&self) -> Entity {
        self.push_children.parent
    }

    /// Adds a command to be executed, like [`Commands::add`].
    pub fn add_command<C: Command + 'static>(&mut self, command: C) -> &mut Self {
        self.commands.add(command);
        self
    }
}

impl<'w> ShapeSpawner<'w> for ShapeChildBuilder<'w> {
    fn spawn_shape(&mut self, bundle: impl Bundle) -> ShapeEntityCommands {
        let Self {
            commands, config, ..
        } = self;
        let mut e = commands.spawn(bundle);
        self.push_children.children.push(e.id());
        if let Some(layers) = &config.render_layers {
            e.insert(layers.clone());
        }
        if let ShapePipelineType::Shape3d = config.pipeline {
            e.insert(Shape3d);
        }

        ShapeEntityCommands {
            commands: e,
            config,
        }
    }

    fn config(&self) -> &ShapeConfig {
        &self.config
    }

    fn set_config(&mut self, config: ShapeConfig) {
        self.config = config;
    }
}

impl<'w> Deref for ShapeChildBuilder<'w> {
    type Target = ShapeConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<'w> DerefMut for ShapeChildBuilder<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

/// Extension trait for [`EntityCommands`] to allow injection of [`ShapeConfig`].
///
/// Useful when parenting shapes under a non-shape entity.
pub trait BuildShapeChildren {
    /// Similar to [`ShapeEntityCommands::with_children`] except is available on non-shape entities, takes in config to pass along to the [`ShapeChildBuilder`]
    fn with_shape_children(
        &mut self,
        config: &ShapeConfig,
        f: impl FnOnce(&mut ShapeChildBuilder),
    ) -> &mut Self;
}

impl<'w> BuildShapeChildren for EntityCommands<'w> {
    fn with_shape_children(
        &mut self,
        config: &ShapeConfig,
        spawn_children: impl FnOnce(&mut ShapeChildBuilder),
    ) -> &mut Self {
        let config = config.without_transform();
        let parent = self.id();
        let mut painter = ShapeChildBuilder {
            commands: self.commands(),
            push_children: PushChildren {
                children: SmallVec::default(),
                parent,
            },
            config,
        };

        spawn_children(&mut painter);
        let children = painter.push_children;
        self.commands().add(children);
        self
    }
}
