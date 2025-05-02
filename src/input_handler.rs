use bevy::prelude::*;

use crate::{input_method_capturing::InputMethodMessage, input_method_data::InputData};

#[derive(Component, Debug)]
#[require(Transform)]
pub struct InputHandler {
    input_events: Vec<InputData>,
    messages: Vec<(Entity, InputMethodMessage)>,
}

impl InputHandler {
    pub const fn new() -> InputHandler {
        InputHandler {
            input_events: Vec::new(),
            messages: Vec::new(),
        }
    }
    // might throw capture_requests in a mutex for better scheduling
    pub fn request_capture(&mut self, method: Entity) {
        self.messages
            .push((method, InputMethodMessage::RequestCapture));
    }
    pub fn release(&mut self, method: Entity) {
        self.messages.push((method, InputMethodMessage::Release));
    }
}

impl InputHandler {
    pub(crate) fn take_messages(&mut self) -> Vec<(Entity, InputMethodMessage)> {
        std::mem::take(&mut self.messages)
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
