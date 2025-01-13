use std::{cmp::Ordering, num::NonZeroU32};

use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    prelude::*,
};

use crate::Field;

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut)]
pub struct RaymarchMaxIterations(pub NonZeroU32);

impl Default for RaymarchMaxIterations {
    fn default() -> Self {
        Self(500.try_into().unwrap())
    }
}

#[derive(Clone, Copy, Component, Debug, Deref)]
pub struct RaymarchDefaultStepSize(pub f32);
impl Default for RaymarchDefaultStepSize {
    fn default() -> Self {
        Self(0.001)
    }
}

// Returns Entities sorted by distance from ray origin
pub fn raymarch_fields(
    ray: &Ray3d,
    fields: Vec<(Entity, &Field, &GlobalTransform)>,
    max_iterations: &RaymarchMaxIterations,
    default_step_size: &RaymarchDefaultStepSize,
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
    max_iterations: &RaymarchMaxIterations,
    min_step_size: &RaymarchDefaultStepSize,
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
        let distance = closest_point.distance(curr_point);
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
