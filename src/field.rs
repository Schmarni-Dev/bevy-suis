use bevy::{
    math::{Vec3A, vec3a},
    prelude::*,
};

pub const RAYMARCH_MAX_STEPS: u32 = 1000;
pub const RAYMARCH_MIN_STEP_SIZE: f32 = 0.001;
pub const RAYMARCH_MAX_DISTANCE: f32 = 10_000.0;

pub struct RayMarchResult {
    pub closest_distance: f32,
    pub deepest_point_ray_length: f32,
    pub ray_lenght: f32,
    pub ray_steps: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum Field {
    Sphere(f32),
    Cuboid(Cuboid),
    Torus(Torus),
    Cylinder(Cylinder),
}
impl Field {
    pub fn closest_point(
        &self,
        field_transform: &GlobalTransform,
        point: impl Into<Vec3A>,
    ) -> Vec3A {
        let point = point.into();
        point - self.normal(field_transform, point) * self.distance(field_transform, point)
    }
    /// point should be in world-space
    pub fn normal(&self, field_transform: &GlobalTransform, point: impl Into<Vec3A>) -> Vec3A {
        let point = point.into();
        let distance_vec = Vec3A::splat(self.distance(field_transform, point));
        const R: f32 = 0.0001;
        let r_vec = Vec3A::new(
            self.distance(field_transform, point + vec3a(R, 0.0, 0.0)),
            self.distance(field_transform, point + vec3a(0.0, R, 0.0)),
            self.distance(field_transform, point + vec3a(0.0, 0.0, R)),
        );
        let local_normal = distance_vec - r_vec;
        -field_transform
            .affine()
            .transform_vector3a(local_normal)
            .normalize()
    }
    /// point should be in world-space
    pub fn distance(&self, field_transform: &GlobalTransform, point: impl Into<Vec3A>) -> f32 {
        let point = point.into();
        let world_to_local_matrix = field_transform.compute_matrix().inverse();
        let p = world_to_local_matrix.transform_point3a(point);
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
            Field::Torus(torus) => {
                let q = vec2(p.xz().length() - torus.major_radius, p.y);
                q.length() - torus.minor_radius
            }
            Field::Cylinder(cylinder) => {
                let d = vec2(
                    p.xz().length().abs() - cylinder.radius,
                    p.y.abs() - cylinder.half_height,
                );
                d.x.max(d.y).min(0.0) + d.max(vec2(0.0, 0.0)).length()
            }
        }
    }
    pub fn raymarch(&self, field_transform: &GlobalTransform, ray: Ray3d) -> RayMarchResult {
        let mut result = RayMarchResult {
            closest_distance: f32::MAX,
            deepest_point_ray_length: 0.,
            ray_lenght: 0.,
            ray_steps: 0,
        };

        while result.ray_steps < RAYMARCH_MAX_STEPS && result.ray_lenght < RAYMARCH_MAX_DISTANCE {
            let point = ray.origin + (ray.direction.as_vec3() * result.ray_lenght);
            let distance = self.distance(field_transform, point);
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
}
