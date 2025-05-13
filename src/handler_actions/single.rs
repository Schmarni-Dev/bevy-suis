use bevy::ecs::{component::Component, entity::Entity};

use crate::{input_handler::InputHandler, input_method_data::InputData};

use super::{DeltaEntitySet, multi::MultiHandlerAction};

#[derive(Component, Default, Debug)]
pub struct SingleHandlerAction {
    multi: MultiHandlerAction,
    actor_started: bool,
    actor_changed: bool,
    actor_stopped: bool,
    actor: Option<Entity>,
}

impl SingleHandlerAction {
    pub fn update(
        &mut self,
        handler: &mut InputHandler,
        allow_changing_actor: bool,
        hover_condition: impl Fn(&InputData) -> bool,
        interact_condition: impl Fn(&InputData) -> bool,
    ) {
        self.multi
            .update(handler, hover_condition, interact_condition);
        self.actor_started = false;
        self.actor_changed = false;
        self.actor_stopped = false;
        if let Some(actor) = self.multi.actor_set().added().iter().next() {
            if self.actor.is_none() {
                self.actor_started = true;
                self.actor = Some(*actor);
            } else if allow_changing_actor {
                self.actor_changed = true;
                self.actor = Some(*actor);
            }
        }

        if let Some(actor) = self.actor.as_mut() {
            if self.multi.actor_set().removed().contains(actor) {
                self.actor_stopped = true;
                self.actor = None;
            }
        }
    }

    pub fn started_acting(&self) -> bool {
        self.actor_started
    }
    pub fn stopped_acting(&self) -> bool {
        self.actor_stopped
    }
    pub fn actor_changed(&self) -> bool {
        self.actor_changed
    }
    pub fn actor(&self, handler: &InputHandler) -> Option<InputData> {
        self.actor
            .and_then(|actor| {
                handler
                    .input_events()
                    .iter()
                    .find(|a| a.input_method == actor)
            })
            .copied()
    }
    pub fn actor_entity(&self) -> Option<Entity> {
        self.actor
    }
}

impl SingleHandlerAction {
    pub fn hover_set(&self) -> &DeltaEntitySet {
        self.multi.hover_set()
    }
    pub fn started_hovering(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        self.multi.started_hovering(handler)
    }
    pub fn currently_hovering(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        self.multi.currently_acting(handler)
    }
    pub fn stopped_hovering(&self, handler: &InputHandler) -> impl Iterator<Item = InputData> {
        self.multi.stopped_acting(handler)
    }
}
