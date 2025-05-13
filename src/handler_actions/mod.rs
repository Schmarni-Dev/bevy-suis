use bevy::ecs::entity::{Entity, EntityHashSet};

pub mod simple;
pub mod multi;
pub mod single;

#[derive(Clone, Default, Debug)]
pub struct DeltaEntitySet {
    added: EntityHashSet,
    current: EntityHashSet,
    removed: EntityHashSet,
}

impl DeltaEntitySet {
    pub const fn new() -> Self {
        Self {
            added: EntityHashSet::new(),
            current: EntityHashSet::new(),
            removed: EntityHashSet::new(),
        }
    }
    pub fn update(&mut self, new_data: impl Iterator<Item = Entity>) {
        let new_set = EntityHashSet::from_iter(new_data);
        self.added = EntityHashSet::from_iter(new_set.difference(&self.current).copied());
        self.removed = EntityHashSet::from_iter(self.current.difference(&new_set).copied());
        self.current = new_set;
    }
    pub fn added(&self) -> &EntityHashSet {
        &self.added
    }
    pub fn current(&self) -> &EntityHashSet {
        &self.current
    }
    pub fn removed(&self) -> &EntityHashSet {
        &self.removed
    }
}
