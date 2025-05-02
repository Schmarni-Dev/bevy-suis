use bevy::prelude::*;

#[derive(Component, Debug, Default)]
#[require(Transform)]
pub struct InputMethod {
    pub captured_by: Option<Entity>,
    handler_order: Vec<Entity>,
}

impl InputMethod {
    pub const fn new() -> InputMethod {
        InputMethod {
            captured_by: None,
            handler_order: Vec::new(),
        }
    }
    pub fn set_handler_order(&mut self, order: Vec<Entity>) {
        self.handler_order = order;
    }
}

impl InputMethod {
    pub(crate) fn get_handler_order(&self) -> &Vec<Entity> {
        &self.handler_order
    }
}
