use bevy::prelude::*;

use crate::input_method_data::{NonSpatialInputData, SpatialInputData};

#[derive(Component, Debug, Default)]
#[require(SpatialInputData, NonSpatialInputData)]
pub struct InputMethod {
    captured_by: Option<Entity>,
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
    pub fn set_captured(&mut self, handler: Entity) {
        self.captured_by = Some(handler);
    }
    pub fn captured_by(&self) -> Option<Entity> {
        self.captured_by
    }
    pub fn release(&mut self) {
        self.captured_by = None;
    }
}

impl InputMethod {
    pub(crate) fn get_handler_order(&self) -> &Vec<Entity> {
        &self.handler_order
    }
}
