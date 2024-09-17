use bevy::{
    app::{Plugin, PreUpdate},
    ecs::{
        component::Component,
        schedule::SystemSet,
        system::{IntoSystem, Query, Resource, System, SystemState},
        world::World,
    },
    log::error,
    math::{Ray3d, Vec3},
    prelude::{App, Entity, IntoSystemConfigs},
    transform::components::{GlobalTransform, Transform},
};
use std::{cmp::Ordering, hash::Hash};
pub mod debug;
pub mod openxr_low_level_actions;
pub mod xr;
pub mod xr_controllers;

pub struct SuisCorePlugin;
impl Plugin for SuisCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                clear_captures,
                run_capture_conditions.in_set(InputMethodCapturingSet),
            )
                .chain(),
        );
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PointerInputMethod(pub Ray3d);

fn clear_captures(mut query: Query<&mut InputMethod>, mut handler_query: Query<&mut InputHandler>) {
    for mut method in &mut query {
        method.captured_by = None;
    }
    for mut handler in &mut handler_query {
        handler.captured_methods.clear();
    }
}

#[derive(SystemSet, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct InputMethodCapturingSet;

fn run_capture_conditions(world: &mut World) {
    let mut state = world
        .remove_resource::<RunCaptureConditionsState>()
        .unwrap_or_else(|| RunCaptureConditionsState(SystemState::new(world)));
    let w = world.as_unsafe_world_cell();
    // SAFETY:
    // idk might be fine, might not be fine
    let (mut method_query, mut handler_query) = unsafe { state.0.get_mut(w.world_mut()) };
    for (method_entity, mut method, method_location) in method_query.iter_mut() {
        let method_position = method_location.translation();
        let mut order = handler_query
            .iter()
            .map(|(e, field, field_location, _)| {
                // add a bias to the forward direction?
                let distance = field.distance2(field_location, method_position);
                (e, distance)
            })
            .collect::<Vec<_>>();
        order.sort_by(
            |(_, distance1), (_, distance2)| match (distance1, distance2) {
                (d1, d2) if d1 > d2 => Ordering::Greater,
                (d1, d2) if d1 < d2 => Ordering::Less,
                (d1, d2) if d1 == d2 => Ordering::Equal,
                (_, _) => {
                    error!("distance1 is not Greater, Less than or Equal to distance2");
                    Ordering::Equal
                }
            },
        );
        let mut iter = handler_query.iter_many_mut(order.into_iter().map(|(e, _)| e));
        while let Some((handler_entity, handler_field, handler_transform, mut handler)) =
            iter.fetch_next()
        {
            // send a precomputed distance?
            let closest_point = handler_transform
                .compute_matrix()
                .inverse()
                .transform_point3(handler_field.closest_point2(handler_transform, method_position));
            // SAFETY:
            // idk might be fine, might not be fine
            handler
                .capture_condition
                .initialize(unsafe { w.world_mut() });
            let wants_to_capture = handler.capture_condition.run(
                CaptureContext {
                    handler: handler_entity,
                    handler_location: *handler_transform,
                    input_method: method_entity,
                    input_method_location: Transform::from_matrix(
                        handler_transform
                            .compute_matrix()
                            .inverse()
                            .mul_mat4(&method_location.compute_matrix()),
                    ),
                    closest_point,
                },
                // SAFETY:
                // idk might be fine, might not be fine
                unsafe { w.world_mut() },
            );
            if wants_to_capture {
                method.captured_by = Some(handler_entity);
                handler.captured_methods.push(method_entity);
                break;
            }
        }
    }

    world.insert_resource(state);
}

#[derive(Component, Debug, Default)]
pub struct InputMethod {
    pub captured_by: Option<Entity>,
}

impl InputMethod {
    pub const fn new() -> InputMethod {
        InputMethod { captured_by: None }
    }
}

#[derive(Component, Debug)]
pub struct InputHandler {
    pub capture_condition: Box<dyn System<In = CaptureContext, Out = bool>>,
    pub captured_methods: Vec<Entity>,
}

impl InputHandler {
    pub fn new<T>(system: impl IntoSystem<CaptureContext, bool, T>) -> InputHandler {
        InputHandler {
            capture_condition: Box::new(IntoSystem::into_system(system)),
            captured_methods: Vec::new(),
        }
    }
}

pub struct CaptureContext {
    pub handler: Entity,
    pub handler_location: GlobalTransform,
    pub input_method: Entity,
    /// Location in handlers local space
    pub input_method_location: Transform,
    /// Point in handlers local space
    pub closest_point: Vec3,
}

#[derive(Component, Debug)]
pub enum Field {
    Sphere(f32),
}
impl Field {
    pub fn closest_point(
        &self,
        this_transform: &Transform,
        reference_space: &Transform,
        point: Vec3,
    ) -> Vec3 {
        let reference_to_this_transform =
            reference_space.compute_matrix().inverse() * this_transform.compute_matrix();
        let local_point = reference_to_this_transform.transform_point3(point);

        let local_closest_point = match self {
            Field::Sphere(r) => local_point.normalize() * (local_point.length().min(*r)),
        };

        reference_to_this_transform
            .inverse()
            .transform_point3(local_closest_point)
    }
    pub fn closest_point2(&self, field_transform: &GlobalTransform, point: Vec3) -> Vec3 {
        let world_to_local_matrix = field_transform.compute_matrix().inverse();
        let local_point = world_to_local_matrix.transform_point3(point);

        let local_closest_point = match self {
            Field::Sphere(r) => local_point.normalize() * (local_point.length().min(*r)),
        };

        world_to_local_matrix
            .inverse()
            .transform_point3(local_closest_point)
    }
    pub fn distance2(&self, field_transform: &GlobalTransform, point: Vec3) -> f32 {
        let closest_point = self.closest_point2(field_transform, point);
        point.distance(closest_point)
    }

    pub fn distance(
        &self,
        this_transform: &Transform,
        reference_space: &Transform,
        point: Vec3,
    ) -> f32 {
        let closest_point = self.closest_point(this_transform, reference_space, point);
        point.distance(closest_point)
    }
}

#[derive(Resource)]
#[allow(clippy::type_complexity)]
struct RunCaptureConditionsState(
    SystemState<(
        Query<'static, 'static, (Entity, &'static mut InputMethod, &'static GlobalTransform)>,
        Query<
            'static,
            'static,
            (
                Entity,
                &'static Field,
                &'static GlobalTransform,
                &'static mut InputHandler,
            ),
        >,
    )>,
);
