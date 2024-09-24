use std::{cmp::Ordering, num::NonZeroU32};

use bevy::{ecs::entity::EntityHashSet, prelude::*};

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

#[derive(Clone, Copy, Component, Debug, Deref)]
pub struct RaymarchHitDistance(pub f32);
impl Default for RaymarchHitDistance {
    fn default() -> Self {
        Self(f32::EPSILON * 4.0)
    }
}

// Returns Entities sorted by distance from ray origin
pub fn raymarch_fields(
    ray: &Ray3d,
    fields: &Query<(Entity, &Field, &GlobalTransform)>,
    max_iterations: &RaymarchMaxIterations,
    hit_distance: &RaymarchHitDistance,
    default_step_size: &RaymarchDefaultStepSize,
) -> Vec<Entity> {
    raymarch(
        ray,
        fields,
        max_iterations,
        hit_distance,
        default_step_size,
        0,
        0.0,
        Vec::new(),
        EntityHashSet::default(),
    )
}

// this is probably very slow, but i don't care for now
#[allow(clippy::too_many_arguments)]
fn raymarch(
    ray: &Ray3d,
    fields: &Query<(Entity, &Field, &GlobalTransform)>,
    max_iterations: &RaymarchMaxIterations,
    hit_distance: &RaymarchHitDistance,
    min_step_size: &RaymarchDefaultStepSize,
    curr_iteration: u32,
    curr_distance: f32,
    mut curr_handlers: Vec<(f32, Entity)>,
    mut hit_handlers: EntityHashSet,
) -> Vec<Entity> {
    if curr_iteration > max_iterations.0.into() {
        return sort_map_vec(curr_handlers);
    }
    // i don't think someone will try to use a pointer over a kilometer
    if curr_distance > 1000.0 {
        return sort_map_vec(curr_handlers);
    }
    let curr_point = ray.get_point(curr_distance);
    let mut step_size = None;
    for (handler, field, field_transform) in fields.iter() {
        if hit_handlers.contains(&handler) {
            continue;
        }
        let closest_point = field.closest_point2(field_transform, curr_point);
        let distance = closest_point.distance(curr_point);
        if step_size.is_none() || step_size.is_some_and(|d| d > distance) {
            step_size = Some(distance);
        }
        if distance <= hit_distance.0 {
            curr_handlers.push((distance, handler));
            hit_handlers.insert(handler);
        }
    }
    raymarch(
        ray,
        fields,
        max_iterations,
        hit_distance,
        min_step_size,
        curr_iteration + 1,
        curr_distance + step_size.unwrap_or(min_step_size.0),
        curr_handlers,
        hit_handlers,
    )
}

fn sort_map_vec(mut vec: Vec<(f32, Entity)>) -> Vec<Entity> {
    vec.sort_by(
        |(distance1, _), (distance2, _)| match (distance1, distance2) {
            (d1, d2) if d1 > d2 => Ordering::Greater,
            (d1, d2) if d1 < d2 => Ordering::Less,
            (d1, d2) if d1 == d2 => Ordering::Equal,
            (_, _) => {
                error!("distance1 is not Greater, Less than or Equal to distance2");
                Ordering::Equal
            }
        },
    );
    vec.into_iter().map(|(_, e)| e).collect()
}