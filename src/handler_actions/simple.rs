use bevy::ecs::component::Component;

use crate::{input_handler::InputHandler, input_method_data::InputData};

use super::DeltaEntitySet;

#[derive(Component, Default, Debug)]
pub struct SimpleHandlerAction {
    actors: DeltaEntitySet,
    wanted_actors: DeltaEntitySet,
}

impl SimpleHandlerAction {
    pub fn update(
        &mut self,
        handler: &mut InputHandler,
        capture_condition: impl Fn(&InputData) -> bool,
    ) {
        self.wanted_actors.update(
            handler
                .input_events()
                .iter()
                .filter(|event| capture_condition(event))
                .map(|event| event.input_method),
        );
        for method in self.wanted_actors.added() {
            handler.request_capture(*method);
        }
        for method in self.wanted_actors.removed() {
            // only releases when actually captured by this handler
            handler.release(*method);
        }
        self.actors.update(
            handler
                .input_events()
                .iter()
                .filter(|event| event.captured)
                .map(|event| event.input_method),
        );
    }
    pub fn actor_set(&self) -> &DeltaEntitySet {
        &self.actors
    }
    pub fn started_acting(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        handler
            .input_events()
            .iter()
            .filter(|event| self.actors.added().contains(&event.input_method))
            .copied()
    }
    pub fn currently_acting(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        handler
            .input_events()
            .iter()
            .filter(|event| self.actors.current().contains(&event.input_method))
            .copied()
    }
    pub fn stopped_acting(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        handler
            .input_events()
            .iter()
            .filter(|event| self.actors.removed().contains(&event.input_method))
            .copied()
    }
}
