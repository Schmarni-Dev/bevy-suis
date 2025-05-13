use bevy::ecs::component::Component;

use crate::{input_handler::InputHandler, input_method_data::InputData};

use super::{DeltaEntitySet, simple::SimpleHandlerAction};

#[derive(Component, Default, Debug)]
pub struct MultiHandlerAction {
    simple: SimpleHandlerAction,
    hovering: DeltaEntitySet,
}

impl MultiHandlerAction {
    pub fn update(
        &mut self,
        handler: &mut InputHandler,
        hover_condition: impl Fn(&InputData) -> bool,
        interact_condition: impl Fn(&InputData) -> bool,
    ) {
        self.simple.update(handler, |data| {
            self.hovering.current().contains(&data.input_method) && interact_condition(data)
        });
        self.hovering.update(
            handler
                .input_events()
                .iter()
                .filter(|data| hover_condition(data))
                .map(|event| event.input_method),
        );
    }

    pub fn actor_set(&self) -> &DeltaEntitySet {
        self.simple.actor_set()
    }
    pub fn hover_set(&self) -> &DeltaEntitySet {
        &self.hovering
    }
    pub fn started_acting(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        self.simple.started_acting(handler)
    }
    pub fn currently_acting(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        self.simple.currently_acting(handler)
    }
    pub fn stopped_acting(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        self.simple.stopped_acting(handler)
    }
    pub fn started_hovering(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        handler
            .input_events()
            .iter()
            .filter(|event| self.hovering.added().contains(&event.input_method))
            .copied()
    }
    pub fn currently_hovering(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        handler
            .input_events()
            .iter()
            .filter(|event| self.hovering.current().contains(&event.input_method))
            .copied()
    }
    pub fn stopped_hovering(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        handler
            .input_events()
            .iter()
            .filter(|event| self.hovering.removed().contains(&event.input_method))
            .copied()
    }
}
