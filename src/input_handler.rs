use bevy::prelude::*;

use crate::{input_method_capturing::InputMethodMessage, input_method_data::InputData};

#[derive(Component, Debug)]
#[require(Transform)]
pub struct InputHandler {
    input_events: Vec<InputData>,
    messages: Vec<(Entity, InputMethodMessage)>,
    field_ref: FieldRef,
}

#[derive(Clone, Copy, Debug)]
pub enum FieldRef {
    This,
    Entity(Entity),
}

impl InputHandler {
    pub const fn new(field_ref: FieldRef) -> InputHandler {
        InputHandler {
            input_events: Vec::new(),
            messages: Vec::new(),
            field_ref,
        }
    }
    pub fn input_events(&self) -> &[InputData] {
        &self.input_events
    }
    // might throw messages in a mutex for better scheduling
    pub fn request_capture(&mut self, method: Entity) {
        self.messages
            .push((method, InputMethodMessage::RequestCapture));
    }
    pub fn release(&mut self, method: Entity) {
        self.messages.push((method, InputMethodMessage::Release));
    }
    pub const fn get_field_ref(&self) -> FieldRef {
        self.field_ref
    }
    pub const fn set_field_ref(&mut self, field_ref: FieldRef) {
        self.field_ref = field_ref;
    }
}

impl InputHandler {
    pub(crate) fn take_messages(&mut self) -> Vec<(Entity, InputMethodMessage)> {
        std::mem::take(&mut self.messages)
    }
    pub(crate) fn set_events(&mut self, events: Vec<InputData>) {
        self.input_events = events;
    }
}
