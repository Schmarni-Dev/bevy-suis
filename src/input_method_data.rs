use bevy::prelude::*;

use crate::{field::Field, hand::Hand};

#[derive(Clone, Copy, Component, Debug, Reflect, Default)]
pub struct NonSpatialInputData {
    /// not available by defualt for hands
    pub scroll: Option<Vec2>,
    /// not available by defualt for hands
    pub pos: Option<Vec2>,
    pub select: f32,
    pub secondary: f32,
    pub context: f32,
    pub grab: f32,
}

#[derive(Clone, Copy, Component, Debug, Reflect)]
#[expect(clippy::large_enum_variant)]
pub enum SpatialInputData {
    Hand(Hand),
    Tip(Isometry3d),
    Ray(Ray3d),
}

impl Default for SpatialInputData {
    fn default() -> Self {
        Self::Tip(Isometry3d::IDENTITY)
    }
}

impl SpatialInputData {
    pub fn transform(self, mat: &Mat4) -> Self {
        match self {
            SpatialInputData::Hand(hand) => SpatialInputData::Hand(hand.transform(mat)),
            SpatialInputData::Tip(isometry) => SpatialInputData::Tip(Isometry3d {
                rotation: mat.to_scale_rotation_translation().1 * isometry.rotation,
                translation: mat.transform_point3a(isometry.translation),
            }),
            SpatialInputData::Ray(ray) => SpatialInputData::Ray(Ray3d {
                origin: mat.transform_point3(ray.origin),
                direction: Dir3::new_unchecked(mat.transform_vector3(ray.direction.as_vec3())),
            }),
        }
    }
    pub fn distance(&self, field: &Field, field_transform: &GlobalTransform) -> f32 {
        match self {
            SpatialInputData::Hand(hand) => hand.distance(field, field_transform),
            SpatialInputData::Tip(isometry) => {
                field.distance(field_transform, isometry.translation)
            }
            SpatialInputData::Ray(ray) => field.raymarch(field_transform, *ray).closest_distance,
        }
    }
    pub fn closest_point(&self, field: &Field, field_transform: &GlobalTransform) -> Vec3A {
        match self {
            SpatialInputData::Hand(hand) => hand.closest_point(field, field_transform),
            SpatialInputData::Tip(isometry) => {
                field.closest_point(field_transform, isometry.translation)
            }
            SpatialInputData::Ray(ray) => field.closest_point(
                field_transform,
                ray.get_point(
                    field
                        .raymarch(field_transform, *ray)
                        .deepest_point_ray_length,
                ),
            ),
        }
    }
    pub fn normal(&self, field: &Field, field_transform: &GlobalTransform) -> Dir3A {
        match self {
            SpatialInputData::Hand(hand) => hand.normal(field, field_transform),
            SpatialInputData::Tip(isometry) => {
                field.normal(field_transform, isometry.translation)
            }
            SpatialInputData::Ray(ray) => field.normal(
                field_transform,
                ray.get_point(
                    field
                        .raymarch(field_transform, *ray)
                        .deepest_point_ray_length,
                ),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputData {
    pub input_method: Entity,
    pub spatial_data: SpatialInputData,
    pub non_spatial_data: NonSpatialInputData,
    pub handler_location: GlobalTransform,
    pub distance: f32,
    pub captured: bool,
}
