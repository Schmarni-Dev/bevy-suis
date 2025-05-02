use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
    transform::systems::{propagate_transforms, sync_simple_transforms},
};

use crate::{
    input_handler::InputHandler,
    input_method::InputMethod,
    input_method_data::{InputData, NonSpatialInputData, SpatialInputData},
    field::Field, SuisPreUpdateSets,
};
pub struct InputMethodCapturingPlugin;

impl Plugin for InputMethodCapturingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                sync_simple_transforms,
                propagate_transforms,
                transfer_input_method_events,
            )
                .in_set(SuisPreUpdateSets::PrepareMethodEvents),
        );
        app.add_systems(
            PreUpdate,
            capture_input_methods.in_set(SuisPreUpdateSets::CaptureInputMethods),
        );
        app.add_systems(
            PreUpdate,
            send_input_data.in_set(SuisPreUpdateSets::SendInputData),
        );
    }
}

fn send_input_data(
    methods: Query<(
        Entity,
        &InputMethod,
        &NonSpatialInputData,
        &GlobalTransform,
        &SpatialInputData,
    )>,
    handlers: Query<(&GlobalTransform, &Field), With<InputHandler>>,
) {
    let mut handler_data = EntityHashMap::<Vec<InputData>>::default();
    for (input_method, method, data, method_transform, input) in &methods {
        if let Some(handler) = method.captured_by {
            let Ok((handler_transform, handler_field)) = handlers
                .get(handler)
                .inspect_err(|err| error!("Invalid InputHandler Capturing InputMethod: {err}"))
            else {
                continue;
            };
            let global_to_handler = handler_transform.compute_matrix().inverse();
            let method_relative_transform = Transform::from_matrix(
                global_to_handler.mul_mat4(&method_transform.compute_matrix()),
            );
            handler_data.entry(handler).or_default().push(InputData {
                input_method,
                input: input.transform(&global_to_handler),
                minimal_non_spatial_data: *data,
                handler_location: *handler_transform,
                input_method_location: method_relative_transform.to_isometry(),
                distance: input.distance(handler_field, handler_transform),
                captured: true,
            });
        }
    }
}

fn capture_input_methods(
    mut methods: Query<(&mut InputMethod, &InputMethodCaptureRequests)>,
    handlers: Query<Has<InputHandler>>,
) {
    for (mut method, capture_requests) in &mut methods {
        if method.captured_by.is_some() {
            continue;
        }
        let iter = method.get_handler_order().clone().into_iter().filter(|e| {
            handlers
                .get(*e)
                .inspect_err(|err| error!("invalid InputHandler in handler_order: {err}"))
                .is_ok_and(|v| v)
        });
        for handler in iter {
            if capture_requests.contains(&handler) {
                method.captured_by = Some(handler);
            }
        }
    }
}

fn transfer_input_method_events(
    mut cmds: Commands,
    mut handlers: Query<(Entity, &mut InputHandler)>,
    mut methods: Query<&mut InputMethod>,
) {
    let mut event_map = EntityHashMap::<InputMethodCaptureRequests>::default();
    for (entity, mut handler) in &mut handlers {
        for (method, msg) in handler.take_messages() {
            match msg {
                InputMethodMessage::RequestCapture => {
                    event_map
                        .entry(method)
                        .or_insert_with(|| InputMethodCaptureRequests(default()))
                        .0
                        .insert(entity);
                }
                InputMethodMessage::Release => {
                    let mut method = match methods.get_mut(method) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Tried to Release an invalid Input Method: {method:?}: {err}");
                            continue;
                        }
                    };
                    method.captured_by.take_if(|e| *e == entity);
                }
            }
        }
    }
    for (method, requests) in event_map.into_iter() {
        if let Err(err) = methods.get(method) {
            error!("Tried Request a Capture for an invalid Input Method: {method:?}: {err}");
            continue;
        }
        cmds.entity(method).insert(requests);
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputMethodMessage {
    RequestCapture,
    Release,
}

#[derive(Debug, PartialEq, Eq, Component, Deref)]
pub struct InputMethodCaptureRequests(EntityHashSet);

#[derive(Clone, Copy, Event)]
pub struct SendInputData {
    pub handler: Entity,
    pub data: InputData,
}
