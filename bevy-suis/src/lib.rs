use bevy::{
    app::{Plugin, PreUpdate},
    ecs::{
        component::Component,
        query::QueryFilter,
        schedule::SystemSet,
        system::{IntoSystem, Query, Resource, System, SystemState},
        world::World,
    },
    log::error,
    math::{Ray3d, Vec3},
    prelude::{App, Cuboid, Deref, Entity, IntoSystemConfigs},
    transform::components::{GlobalTransform, Transform},
};
use raymarching::{
    raymarch_fields, RaymarchDefaultStepSize, RaymarchHitDistance, RaymarchMaxIterations,
};
use std::{cmp::Ordering, hash::Hash};
pub mod debug;
pub mod raymarching;
pub mod window_pointers;
pub mod xr;
pub mod xr_controllers;

pub struct SuisCorePlugin;
impl Plugin for SuisCorePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            (
                SuisPreUpdateSets::UpdateInputMethods,
                SuisPreUpdateSets::InputMethodCapturing,
            ),
        );
        app.add_systems(
            PreUpdate,
            (clear_captures, run_capture_conditions)
                .chain()
                .in_set(SuisPreUpdateSets::InputMethodCapturing),
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
pub enum SuisPreUpdateSets {
    UpdateInputMethods,
    InputMethodCapturing,
}

pub fn pipe_input_ctx<HandlerFilter: QueryFilter>(
    query: Query<(Entity, &Field, &GlobalTransform, &InputHandler), HandlerFilter>,
    methods_query: Query<(
        &GlobalTransform,
        Option<(
            &PointerInputMethod,
            Option<&RaymarchMaxIterations>,
            Option<&RaymarchDefaultStepSize>,
            Option<&RaymarchHitDistance>,
        )>,
    )>,
) -> Vec<InputHandlingContext> {
    let mut out = Vec::new();
    for (handler, field, handler_transform, handler_data) in query.iter() {
        let mut methods = Vec::new();
        for (method, (method_location, pointer_method)) in handler_data
            .captured_methods
            .iter()
            .filter_map(|e| methods_query.get(*e).map(|v| (*e, v)).ok())
        {
            let point = match pointer_method {
                None => method_location.translation(),
                Some((ray, max_iters, min_step_size, hit_distance)) => {
                    raymarch_fields(
                        &ray.0,
                        vec![(handler, field, handler_transform)],
                        max_iters.unwrap_or(&Default::default()),
                        hit_distance.unwrap_or(&Default::default()),
                        min_step_size.unwrap_or(&Default::default()),
                    )
                    .iter()
                    .find(|(_, e)| *e == handler)
                    .map(|(p, _)| *p);
                    todo!()
                }
            };
            // TODO: make this a better default for hands and pointers
            let closest_point =
                field.closest_point2(handler_transform, method_location.translation());
            methods.push(InnerInputHandlingContext {
                input_method: method,
                input_method_location: Transform::from_matrix(
                    handler_transform
                        .compute_matrix()
                        .inverse()
                        .mul_mat4(&method_location.compute_matrix()),
                ),
                closest_point,
            });
        }
        out.push(InputHandlingContext {
            handler,
            handler_location: *handler_transform,
            methods,
        })
    }

    out
}

#[derive(Clone, Copy, Debug, Component, Deref)]
pub struct PreviousInputMehtodData(pub Entity);

fn run_capture_conditions(world: &mut World) {
    let mut state = world
        .remove_resource::<RunCaptureConditionsState>()
        .unwrap_or_else(|| RunCaptureConditionsState(SystemState::new(world)));
    // SAFETY:
    // NOT FINE! let's hope no one despawns a handler or method, or modifies any of the components
    // that we reference
    let w = world.as_unsafe_world_cell();
    let (mut method_query, mut handler_query) = unsafe { state.0.get_mut(w.world_mut()) };
    for (method_entity, mut method, method_location, ray_method) in method_query.iter_mut() {
        let method_position = method_location.translation();
        let order = if let Some((ray, max_iters, min_step_size, hit_distance)) = ray_method {
            raymarch_fields(
                &ray.0,
                handler_query.iter().map(|(e, f, t, _)| (e, f, t)).collect(),
                max_iters.unwrap_or(&Default::default()),
                hit_distance.unwrap_or(&Default::default()),
                min_step_size.unwrap_or(&Default::default()),
            )
        } else {
            let mut o = handler_query
                .iter()
                .map(|(e, field, field_location, _)| {
                    let point = field.closest_point2(field_location, method_position);

                    let distance = point.distance(method_position);

                    (e, distance, point)
                })
                .collect::<Vec<_>>();
            o.sort_by(|(_, distance1, _), (_, distance2, _)| {
                distance1.partial_cmp(distance2).unwrap_or(Ordering::Equal)
            });
            o.into_iter().map(|(e, _, p)| (p, e)).collect()
        };
        for (point, handler_entity) in order.into_iter() {
            let Ok((handler_entity, handler_field, handler_transform, mut handler)) =
                handler_query.get_mut(handler_entity)
            else {
                continue;
            };
            // send a precomputed distance?
            let closest_point = handler_transform
                .compute_matrix()
                .inverse()
                .transform_point3(point);
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
pub struct InputHandlingContext {
    pub handler: Entity,
    pub handler_location: GlobalTransform,
    pub methods: Vec<InnerInputHandlingContext>,
}

pub struct InnerInputHandlingContext {
    pub input_method: Entity,
    /// Location in handlers local space
    pub input_method_location: Transform,
    /// Point in handlers local space
    pub closest_point: Vec3,
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
    Cuboid(Cuboid),
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
            Field::Cuboid(cuboid) => cuboid.closest_point(local_point),
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
            Field::Cuboid(cuboid) => cuboid.closest_point(local_point),
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
        Query<
            'static,
            'static,
            (
                Entity,
                &'static mut InputMethod,
                &'static GlobalTransform,
                Option<(
                    &'static PointerInputMethod,
                    Option<&'static RaymarchMaxIterations>,
                    Option<&'static RaymarchDefaultStepSize>,
                    Option<&'static RaymarchHitDistance>,
                )>,
            ),
        >,
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
