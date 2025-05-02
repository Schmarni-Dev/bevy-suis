use bevy::prelude::*;

use crate::Field;


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
