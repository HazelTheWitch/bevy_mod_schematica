use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    system::Commands,
    world::{Command, World},
};
use bevy_hierarchy::BuildWorldChildren;
use smallvec::{smallvec, SmallVec};

pub mod hierarchy;
pub mod schematics;

#[derive(Debug, Clone, Copy)]
pub struct EntityInfo {
    pub(crate) entity: Entity,
    pub(crate) parent: Option<usize>,
}

impl EntityInfo {
    pub(crate) const fn new(entity: Entity, parent: Option<usize>) -> Self {
        Self { entity, parent }
    }

    pub(crate) const fn root(entity: Entity) -> Self {
        Self {
            entity,
            parent: None,
        }
    }

    pub const fn entity(&self) -> Entity {
        self.entity
    }

    pub const fn parent(&self) -> Option<usize> {
        self.parent
    }
}

pub struct SchematicContext<'l> {
    pub(crate) world: &'l mut World,
    pub(crate) entities: &'l mut SmallVec<[EntityInfo; 1]>,
    pub(crate) current_entity: usize,
}

impl<'l> SchematicContext<'l> {
    pub fn root(&self) -> EntityInfo {
        *self.entities.get(0).unwrap_or_else(|| unreachable!())
    }

    pub fn current(&self) -> EntityInfo {
        *self
            .entities
            .get(self.current_entity)
            .unwrap_or_else(|| unreachable!())
    }

    pub fn get(&self, index: usize) -> Option<EntityInfo> {
        self.entities.get(index).copied()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn entities(&self) -> &SmallVec<[EntityInfo; 1]> {
        &self.entities
    }

    pub fn current_entity(&self) -> usize {
        self.current_entity
    }

    pub fn set_current(&mut self, index: usize) -> Option<()> {
        if index >= self.len() {
            return None;
        }

        self.current_entity = index;

        Some(())
    }

    pub fn insert<T: Bundle>(&mut self, bundle: T) {
        self.world.entity_mut(self.current().entity).insert(bundle);
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn of<'s>(&'s mut self, index: usize) -> Option<SchematicContext<'s>> {
        if index >= self.len() {
            return None;
        }

        Some(SchematicContext {
            world: self.world,
            entities: self.entities,
            current_entity: index,
        })
    }

    pub fn with<'s, T>(
        &'s mut self,
        index: usize,
        f: impl FnOnce(SchematicContext<'s>) -> T,
    ) -> Option<T> {
        Some(f(self.of(index)?))
    }

    pub fn with_child<'s, T>(&'s mut self, f: impl FnOnce(SchematicContext<'s>) -> T) -> T {
        let child = self.world.spawn_empty().id();

        self.entities
            .push(EntityInfo::new(child, Some(self.current_entity)));
        self.world
            .entity_mut(self.current().entity)
            .add_child(child);

        let child_index = self.entities.len() - 1;

        self.with(child_index, f).unwrap_or_else(|| unreachable!())
    }

    pub fn map_children<I, T>(
        &mut self,
        children: impl Iterator<Item = I>,
        mut f: impl FnMut(SchematicContext<'_>, I) -> T,
    ) -> Vec<T> {
        let mut outputs = Vec::new();

        for child in children {
            outputs.push(self.with_child(|ctx| f(ctx, child)));
        }

        outputs
    }

    pub fn try_map_children<I, T, E>(
        &mut self,
        children: impl Iterator<Item = I>,
        mut f: impl FnMut(SchematicContext<'_>, I) -> Result<T, E>,
    ) -> Result<Vec<T>, E> {
        let mut outputs = Vec::new();

        for child in children {
            outputs.push(self.with_child(|ctx| f(ctx, child))?);
        }

        Ok(outputs)
    }

    pub fn children(&self) -> SmallVec<[usize; 1]> {
        self.entities
            .iter()
            .enumerate()
            .filter_map(|(index, info)| {
                if info.parent == Some(self.current_entity) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct SchematicError(&'static str);

impl Display for SchematicError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for SchematicError {}

pub type SchematicResult = Result<(), SchematicError>;

pub trait Schematic: Send + 'static {
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult;
}

impl<C> Schematic for C
where
    C: Component + Send + 'static,
{
    fn instantiate(self, ctx: &mut SchematicContext) -> SchematicResult {
        ctx.insert(self);

        Ok(())
    }
}

pub trait SchematicWorldExt {
    fn spawn_schematic(&mut self, schematic: impl Schematic) -> Result<Entity, SchematicError>;
}

impl SchematicWorldExt for World {
    fn spawn_schematic(&mut self, schematic: impl Schematic) -> Result<Entity, SchematicError> {
        let mut entities = smallvec![EntityInfo::root(self.spawn_empty().id())];

        let mut ctx = SchematicContext {
            world: self,
            entities: &mut entities,
            current_entity: 0,
        };

        schematic.instantiate(&mut ctx)?;

        Ok(ctx.current().entity)
    }
}

pub struct SchematicCommand<S>(pub S);

impl<S> Command for SchematicCommand<S>
where
    S: Schematic,
{
    fn apply(self, world: &mut World) {
        world
            .spawn_schematic(self.0)
            .expect("schematic spawning failed");
    }
}

pub trait SchematicCommandsExt {
    fn spawn_schematic(&mut self, schematic: impl Schematic);
}

impl<'w, 's> SchematicCommandsExt for Commands<'w, 's> {
    fn spawn_schematic(&mut self, schematic: impl Schematic) {
        self.add(SchematicCommand(schematic));
    }
}

pub mod prelude {
    pub use crate::{
        hierarchy::{Children, Parent},
        schematics::{Many, Maybe, OrDefault},
        Schematic, SchematicCommandsExt, SchematicContext, SchematicError, SchematicResult,
        SchematicWorldExt,
    };
    pub use bevy_mod_schematica_macros::Schematic;
}
