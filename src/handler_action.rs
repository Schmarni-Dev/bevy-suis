use bevy::{ecs::entity::EntityHashSet, prelude::*};

use crate::{input_handler::InputHandler, input_method_data::InputData};

pub struct SimpleHandlerAction {
    new_actors: EntityHashSet,
    current_actors: EntityHashSet,
    old_actors: EntityHashSet,
}

impl SimpleHandlerAction {
    pub fn update(
        &mut self,
        handler: &mut InputHandler,
        capture_condition: impl Fn(&InputData) -> bool,
    ) {
        let curr_actors =
            EntityHashSet::from_iter(handler.input_events().iter().map(|e| e.input_method));
        self.new_actors.clear();
        for actor in &curr_actors {
            if !self.current_actors.contains(actor) {
                self.new_actors.insert(*actor);
            }
        }
        self.old_actors = std::mem::replace(&mut self.current_actors, curr_actors);
        self.old_actors.retain(|a| !self.current_actors.contains(a));
        let wants_capture = handler
            .input_events()
            .iter()
            .map(|event| (event.input_method, capture_condition(event)))
            .collect::<Vec<_>>();
        for (method, wants_capture) in wants_capture {
            match wants_capture {
                true if !self.current_actors.contains(&method) => handler.request_capture(method),
                false if self.current_actors.contains(&method) => handler.release(method),
                _ => {}
            }
        }
    }
}
