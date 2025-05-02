use std::{cmp::Ordering, num::NonZeroU32};

use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
};

use crate::Field;

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut)]
pub struct RaymarchMaxSteps(pub NonZeroU32);
impl Default for RaymarchMaxSteps {
    fn default() -> Self {
        Self(500.try_into().unwrap())
    }
}
pub const RAYMARCH_MAX_STEPS: u32 = 1000;
pub const RAYMARCH_MIN_STEP_SIZE: f32 = 0.001;
pub const RAYMARCH_MAX_DISTANCE: f32 = 10_000.0;

#[derive(Clone, Copy, Component, Debug, Deref)]
pub struct RaymarchMinStepSize(pub f32);
impl Default for RaymarchMinStepSize {
    fn default() -> Self {
        Self(0.001)
    }
}

#[derive(Clone, Copy, Component, Debug, Deref)]
pub struct RaymarchMaxDistance(pub f32);
impl Default for RaymarchMaxDistance {
    fn default() -> Self {
        Self(1_000.0)
    }
}

pub struct RayMarchResult {
    pub closest_distance: f32,
    pub deepest_point_ray_length: f32,
    pub ray_lenght: f32,
    pub ray_steps: u32,
}

pub fn raymarch_field(
    ray: Ray3d,
    field: &Field,
    field_transform: &GlobalTransform,
) -> RayMarchResult {
    let mut result = RayMarchResult {
        closest_distance: f32::MAX,
        deepest_point_ray_length: 0.,
        ray_lenght: 0.,
        ray_steps: 0,
    };

    while result.ray_steps < RAYMARCH_MAX_STEPS && result.ray_lenght < RAYMARCH_MAX_DISTANCE {
        let point = ray.origin + (ray.direction.as_vec3() * result.ray_lenght);
        let distance = field.distance(field_transform, point);
        if distance < result.closest_distance {
            result.closest_distance = distance;
            result.deepest_point_ray_length = result.ray_lenght;
        }
        // max_distance isn't meant for this but it doesn't make sense to march further than the
        // limit
        result.ray_lenght += distance.max(RAYMARCH_MIN_STEP_SIZE);
    }

    result
}

// Returns Entities sorted by distance from ray origin
pub fn raymarch_fields(
    ray: &Ray3d,
    fields: Vec<(Entity, &Field, &GlobalTransform)>,
    max_iterations: &RaymarchMaxSteps,
    default_step_size: &RaymarchMinStepSize,
) -> Vec<(Vec3, Entity)> {
    raymarch(
        ray,
        fields,
        max_iterations,
        default_step_size,
        0,
        0.0,
        EntityHashMap::default(),
        EntityHashSet::default(),
    )
}

struct RaymarchData {
    distance_on_ray: f32,
    distance_to_closest_point: f32,
    point_on_ray: Vec3,
}

// this is probably very slow, but i don't care for now
#[allow(clippy::too_many_arguments)]
fn raymarch(
    ray: &Ray3d,
    fields: Vec<(Entity, &Field, &GlobalTransform)>,
    max_iterations: &RaymarchMaxSteps,
    min_step_size: &RaymarchMinStepSize,
    curr_iteration: u32,
    curr_distance: f32,
    mut curr_handlers: EntityHashMap<RaymarchData>,
    mut hit_handlers: EntityHashSet,
) -> Vec<(Vec3, Entity)> {
    if curr_iteration > max_iterations.0.into() {
        return get_final_vec(curr_handlers);
    }
    // i don't think someone will try to use a pointer over 10 kilometers
    if curr_distance > 10000.0 {
        return get_final_vec(curr_handlers);
    }
    let curr_point = ray.get_point(curr_distance);
    let mut step_size = None;
    for (handler, field, field_transform) in fields.iter() {
        if hit_handlers.contains(handler) {
            continue;
        }
        let closest_point = field.closest_point(field_transform, curr_point);
        let distance = closest_point.distance(curr_point.into());
        if step_size.is_none() || step_size.is_some_and(|d| d > distance) {
            step_size = Some(distance);
        }

        let smallest_distance = curr_handlers
            .get(handler)
            .map(|v| v.distance_to_closest_point)
            .unwrap_or(f32::MAX);
        if distance <= smallest_distance {
            curr_handlers.insert(
                *handler,
                RaymarchData {
                    distance_on_ray: curr_distance,
                    distance_to_closest_point: distance,
                    point_on_ray: curr_point,
                },
            );
        }
        if distance < 0.0 {
            hit_handlers.insert(*handler);
        }
    }
    raymarch(
        ray,
        fields,
        max_iterations,
        min_step_size,
        curr_iteration + 1,
        curr_distance + step_size.unwrap_or(min_step_size.0),
        curr_handlers,
        hit_handlers,
    )
}

fn get_final_vec(map: EntityHashMap<RaymarchData>) -> Vec<(Vec3, Entity)> {
    let mut vec: Vec<_> = map.into_iter().collect();
    vec.sort_by(|(_, data1), (_, data2)| {
        match data1
            .distance_on_ray
            .partial_cmp(&data2.distance_on_ray)
            .unwrap_or(Ordering::Equal)
        {
            Ordering::Equal => data1
                .distance_to_closest_point
                .partial_cmp(&data2.distance_to_closest_point)
                .unwrap_or(Ordering::Equal),
            a => a,
        }
    });
    vec.into_iter()
        .map(|(e, data)| (data.point_on_ray, e))
        .collect()
}
