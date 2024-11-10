use bevy::prelude::*;
use bevy::{
    ecs::{
        entity::{EntityHashMap, EntityHashSet},
        query::QueryFilter,
        system::{System, SystemState},
    },
    math::vec3,
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
#[derive(Deref, DerefMut, Debug, Clone, Copy, Component)]
pub struct InputMethodActive(pub bool);

#[derive(Deref, DerefMut, Debug, Clone, Copy, Component)]
pub struct PointerInputMethod(pub Ray3d);

fn clear_captures(
    mut query: Query<(Entity, &mut InputMethod, Option<&mut LastCapturedBy>)>,
    mut handler_query: Query<&mut InputHandlerCaptures>,
    mut cmds: Commands,
) {
    for (e, mut method, last) in &mut query {
        let last_captured = method.captured_by.take();
        match last {
            Some(mut v) => v.0 = last_captured,
            None => {
                cmds.entity(e).insert(LastCapturedBy(last_captured));
            }
        }
    }
    for mut handler in &mut handler_query {
        handler.captured_methods.clear();
    }
}

#[derive(Component, Clone, Copy, Default)]
struct LastCapturedBy(Option<Entity>);

#[derive(SystemSet, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum SuisPreUpdateSets {
    UpdateInputMethods,
    InputMethodCapturing,
}

pub fn pipe_input_ctx<HandlerFilter: QueryFilter>(
    query: Query<(Entity, &Field, &GlobalTransform, &InputHandlerCaptures), HandlerFilter>,
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
                Some((ray, max_iters, min_step_size, hit_distance)) => raymarch_fields(
                    &ray.0,
                    vec![(handler, field, handler_transform)],
                    max_iters.unwrap_or(&Default::default()),
                    hit_distance.unwrap_or(&Default::default()),
                    min_step_size.unwrap_or(&Default::default()),
                )
                .iter()
                .find(|(_, e)| *e == handler)
                .map(|(p, _)| *p)
                .unwrap_or(method_location.translation()),
            };
            let point_local = handler_transform
                .compute_matrix()
                .inverse()
                .transform_point3(point);
            // TODO: make this a better default for hands
            let closest_point = handler_transform
                .compute_matrix()
                .inverse()
                .transform_point3(field.closest_point(handler_transform, point));
            methods.push(InnerInputHandlingContext {
                input_method: method,
                input_method_location: Transform::from_matrix(
                    handler_transform
                        .compute_matrix()
                        .inverse()
                        .mul_mat4(&method_location.compute_matrix()),
                )
                .with_translation(point_local),
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
    let mut insert_active = EntityHashSet::default();
    let (mut method_query, handler_query) = state.0.get_mut(world);
    let mut interactions: EntityHashMap<Vec<(Vec3, Entity)>> = default();
    for (method_entity, method_location, last_captured_by, active, ray_method) in
        method_query.iter_mut()
    {
        let method_position = method_location.translation();
        match active {
            Some(v) => {
                if !v.0 {
                    continue;
                }
            }
            None => {
                insert_active.insert(method_entity);
            }
        }
        let mut order = if let Some((ray, max_iters, min_step_size, hit_distance)) = ray_method {
            raymarch_fields(
                &ray.0,
                handler_query.iter().collect(),
                max_iters.unwrap_or(&Default::default()),
                hit_distance.unwrap_or(&Default::default()),
                min_step_size.unwrap_or(&Default::default()),
            )
        } else {
            let mut o = handler_query
                .iter()
                .map(|(e, field, field_location)| {
                    let point = field.closest_point(field_location, method_position);

                    let distance = point.distance(method_position);

                    (e, distance, method_position)
                })
                .collect::<Vec<_>>();
            o.sort_by(|(_, distance1, _), (_, distance2, _)| {
                distance1.partial_cmp(distance2).unwrap_or(Ordering::Equal)
            });
            o.into_iter().map(|(e, _, p)| (p, e)).collect()
        };
        if let Some(last) = last_captured_by.0 {
            if let Some(index) = order
                .iter()
                .enumerate()
                .find(|(_, v)| v.1 == last)
                .map(|(i, _)| i)
            {
                let data = order.remove(index);
                order.insert(0, data);
            } else {
                order.insert(0, (method_position, last));
            }
        }
        interactions.insert(method_entity, order);
    }
    for e in insert_active.into_iter() {
        world.entity_mut(e).insert(InputMethodActive(true));
    }
    for (method_entity, order) in interactions.into_iter() {
        fn x(world: &mut World, entity: Entity) -> Option<(Entity, InputMethod, GlobalTransform)> {
            let mut e = world.get_entity_mut(entity)?;
            Some((entity, e.take()?, e.get().copied()?))
        }
        let Some((method_entity, mut method, method_location)) = x(world, method_entity) else {
            continue;
        };
        for (point, handler_entity) in order.into_iter() {
            fn x(
                world: &mut World,
                entity: Entity,
            ) -> Option<(Entity, Field, GlobalTransform, InputHandler)> {
                let mut e = world.get_entity_mut(entity)?;
                Some((
                    entity,
                    e.get::<Field>().copied()?,
                    e.get::<GlobalTransform>().copied()?,
                    e.take::<InputHandler>()?,
                ))
            }
            let Some((handler_entity, handler_field, handler_transform, mut handler)) =
                x(world, handler_entity)
            else {
                continue;
            };
            let closest_point = handler_transform
                .compute_matrix()
                .inverse()
                .transform_point3(handler_field.closest_point(&handler_transform, point));
            let distance = handler_field.distance(&handler_transform, point);
            // send a precomputed distance?
            let point = handler_transform
                .compute_matrix()
                .inverse()
                .transform_point3(point);
            handler.capture_condition.initialize(world);
            let wants_to_capture = handler.capture_condition.run(
                CaptureContext {
                    handler: handler_entity,
                    handler_location: handler_transform,
                    input_method: method_entity,
                    input_method_location: Transform::from_matrix(
                        handler_transform
                            .compute_matrix()
                            .inverse()
                            .mul_mat4(&method_location.compute_matrix()),
                    )
                    .with_translation(point),
                    closest_point,
                    distance
                },
                world,
            );
            let mut e = world.entity_mut(handler_entity);
            let mut captures = e.take::<InputHandlerCaptures>().unwrap_or_default();
            if wants_to_capture {
                method.captured_by = Some(handler_entity);
                captures.captured_methods.push(method_entity);
            }
            e.insert(captures);
            e.insert(handler);
            if wants_to_capture {
                break;
            }
        }
        world.entity_mut(method_entity).insert(method);
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

#[derive(Component, Debug, Default)]
pub struct InputHandlerCaptures {
    pub captured_methods: Vec<Entity>,
}

#[derive(Component, Debug)]
pub struct InputHandler {
    pub capture_condition: Box<dyn System<In = CaptureContext, Out = bool>>,
}

impl InputHandler {
    pub fn new<T>(system: impl IntoSystem<CaptureContext, bool, T>) -> InputHandler {
        InputHandler {
            capture_condition: Box::new(IntoSystem::into_system(system)),
        }
    }
}

pub struct CaptureContext {
    pub handler: Entity,
    pub handler_location: GlobalTransform,
    pub input_method: Entity,
    /// Location in handlers local space
    pub input_method_location: Transform,
    /// Closest Point the the surface of the field, Point is in handlers local space
    pub closest_point: Vec3,
    /// Signed Distance between the input method and the surface of the field
    pub distance: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum Field {
    Sphere(f32),
    Cuboid(Cuboid),
}
impl Field {
    pub fn closest_point(&self, field_transform: &GlobalTransform, point: Vec3) -> Vec3 {
        point - self.normal(field_transform, point) * self.distance(field_transform, point)
    }
    /// point should be in world-space
    pub fn normal(&self, field_transform: &GlobalTransform, point: Vec3) -> Vec3 {
        let distance_vec = Vec3::splat(self.distance(field_transform, point));
        const R: f32 = 0.0001;
        const G: &'static GlobalTransform = &GlobalTransform::IDENTITY;
        let r_vec = Vec3::new(
            self.distance(field_transform, point + vec3(R, 0.0, 0.0)),
            self.distance(field_transform, point + vec3(0.0, R, 0.0)),
            self.distance(field_transform, point + vec3(0.0, 0.0, R)),
        );
        let local_normal = distance_vec - r_vec;
        -field_transform
            .affine()
            .transform_vector3(local_normal)
            .normalize()
    }
    /// point should be in world-space
    pub fn distance(&self, field_transform: &GlobalTransform, point: Vec3) -> f32 {
        let world_to_local_matrix = field_transform.compute_matrix().inverse();
        let p = world_to_local_matrix.transform_point3(point);
        match self {
            Field::Sphere(radius) => p.length() - radius,
            Field::Cuboid(cuboid) => {
                let q = Vec3::new(
                    p.x.abs() - cuboid.half_size.x,
                    p.y.abs() - cuboid.half_size.y,
                    p.z.abs() - cuboid.half_size.z,
                );
                let v = Vec3::new(q.x.max(0_f32), q.y.max(0_f32), q.z.max(0_f32));
                v.length() + q.x.max(q.y.max(q.z)).min(0_f32)
            }
        }
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
                &'static GlobalTransform,
                &'static LastCapturedBy,
                Option<&'static InputMethodActive>,
                Option<(
                    &'static PointerInputMethod,
                    Option<&'static RaymarchMaxIterations>,
                    Option<&'static RaymarchDefaultStepSize>,
                    Option<&'static RaymarchHitDistance>,
                )>,
            ),
        >,
        Query<'static, 'static, (Entity, &'static Field, &'static GlobalTransform)>,
    )>,
);
