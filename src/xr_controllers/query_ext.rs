use bevy::{math::Vec2, prelude::{Entity, Query}};
use schminput::{subaction_paths::SubactionPath, BoolActionValue, F32ActionValue, Vec2ActionValue};


pub(super) trait ActionValueQueryExt {
    type ReturnType;
    fn get_with_path_or_default(&self, entity: Entity, path: &SubactionPath) -> Self::ReturnType;
}

impl ActionValueQueryExt for Query<'_, '_, &BoolActionValue> {
    type ReturnType = bool;

    fn get_with_path_or_default(&self, entity: Entity, path: &SubactionPath) -> Self::ReturnType {
        self.get(entity)
            .ok()
            .and_then(|v| v.get_with_path(path))
            .copied()
            .unwrap_or_default()
    }
}
impl ActionValueQueryExt for Query<'_, '_, &F32ActionValue> {
    type ReturnType = f32;

    fn get_with_path_or_default(&self, entity: Entity, path: &SubactionPath) -> Self::ReturnType {
        self.get(entity)
            .ok()
            .and_then(|v| v.get_with_path(path))
            .copied()
            .unwrap_or_default()
    }
}
impl ActionValueQueryExt for Query<'_, '_, &Vec2ActionValue> {
    type ReturnType = Vec2;

    fn get_with_path_or_default(&self, entity: Entity, path: &SubactionPath) -> Self::ReturnType {
        self.get(entity)
            .ok()
            .and_then(|v| v.get_with_path(path))
            .copied()
            .unwrap_or_default()
    }
}
