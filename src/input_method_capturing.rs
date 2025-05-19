use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
    transform::systems::{propagate_parent_transforms, sync_simple_transforms},
};

use crate::{
    SuisPreUpdateSets,
    field::Field,
    input_handler::{FieldRef, InputHandler},
    input_method::InputMethod,
    input_method_data::{InputData, NonSpatialInputData, SpatialInputData},
};
pub struct InputMethodCapturingPlugin;

impl Plugin for InputMethodCapturingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                sync_simple_transforms,
                propagate_parent_transforms,
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
        &SpatialInputData,
    )>,
    mut handlers: Query<(Entity, &GlobalTransform, &mut InputHandler)>,
    field_query: Query<(&Field, &GlobalTransform)>,
) {
    let mut handler_data = EntityHashMap::<Vec<InputData>>::default();
    for (input_method, method, data, input) in &methods {
        if let Some(handler) = method.captured_by() {
            let Ok((handler, handler_transform, input_handler)) = handlers
                .get(handler)
                .inspect_err(|err| error!("Invalid InputHandler Capturing InputMethod: {err}"))
            else {
                continue;
            };
            let Some(data) = get_data_for_handler(
                handler,
                handler_transform,
                input_handler,
                field_query,
                |global_to_handler, field, field_transform| InputData {
                    input_method,
                    spatial_data: input.transform(&global_to_handler),
                    non_spatial_data: *data,
                    handler_location: *handler_transform,
                    distance: input.distance(field, field_transform),
                    captured: true,
                },
            ) else {
                continue;
            };
            handler_data.entry(handler).or_default().push(data);
        } else {
            for (handler, handler_transform, input_handler) in &handlers {
                let Some(data) = get_data_for_handler(
                    handler,
                    handler_transform,
                    input_handler,
                    field_query,
                    |global_to_handler, field, field_transform| InputData {
                        input_method,
                        spatial_data: input.transform(&global_to_handler),
                        non_spatial_data: *data,
                        handler_location: *handler_transform,
                        distance: input.distance(field, field_transform),
                        captured: false,
                    },
                ) else {
                    continue;
                };
                handler_data.entry(handler).or_default().push(data);
            }
        }
    }

    for (handler, _, mut input_handler) in &mut handlers {
        let data = handler_data.remove(&handler).unwrap_or_default();
        input_handler.set_events(data);
    }
}

fn get_data_for_handler(
    handler: Entity,
    handler_transform: &GlobalTransform,
    input_handler: &InputHandler,
    field_query: Query<(&Field, &GlobalTransform)>,
    creation_fn: impl FnOnce(Mat4, &Field, &GlobalTransform) -> InputData,
) -> Option<InputData> {
    let global_to_handler = handler_transform.compute_matrix().inverse();
    let field_entity = match input_handler.get_field_ref() {
        FieldRef::This => handler,
        FieldRef::Entity(entity) => entity,
    };
    let Ok((field, field_transform)) = field_query
        .get(field_entity)
        .inspect_err(|err| error!("Invalid Field: {err}"))
    else {
        return None;
    };
    Some(creation_fn(global_to_handler, field, field_transform))
}

fn capture_input_methods(
    mut methods: Query<(&mut InputMethod, &InputMethodCaptureRequests)>,
    handlers: Query<Has<InputHandler>>,
) {
    for (mut method, capture_requests) in &mut methods {
        if method.captured_by().is_some() {
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
                method.set_captured(handler);
            }
        }
    }
}

fn transfer_input_method_events(
    mut cmds: Commands,
    mut handlers: Query<(Entity, &mut InputHandler)>,
    mut methods: Query<(Entity, &mut InputMethod)>,
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
                    let (_, mut method) = match methods.get_mut(method) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("Tried to Release an invalid Input Method: {method:?}: {err}");
                            continue;
                        }
                    };
                    if method.captured_by() == Some(entity) {
                        method.release();
                    }
                }
            }
        }
    }
    for (method, _) in &methods {
        let Some(requests) = event_map.remove(&method) else {
            cmds.entity(method).remove::<InputMethodCaptureRequests>();
            continue;
        };
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
