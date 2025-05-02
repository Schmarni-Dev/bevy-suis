use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    field::Field,
    input_handler::{FieldRef, InputHandler},
};

#[derive(SystemParam)]
pub struct InputHandlerQueryHelper<'w, 's> {
    handler_query: Query<'w, 's, (Entity, &'static InputHandler), With<InputHandler>>,
    field_query: Query<'w, 's, (&'static Field, &'static GlobalTransform)>,
}
impl InputHandlerQueryHelper<'_, '_> {
    pub fn query_all_handler_fields<T>(
        &self,
        callback: impl Fn((Entity, &Field, &GlobalTransform)) -> T,
    ) -> Vec<T> {
        self.handler_query
            .iter()
            .map(|(entity, handler)| {
                (
                    entity,
                    match handler.get_field_ref() {
                        FieldRef::This => entity,
                        FieldRef::Entity(entity) => entity,
                    },
                )
            })
            .filter_map(|(handler, field)| {
                self.field_query
                    .get(field)
                    .map(|(field, transform)| (handler, field, transform))
                    .ok()
            })
            .map(callback)
            .collect::<Vec<T>>()
    }
}
