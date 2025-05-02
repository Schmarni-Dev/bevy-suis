use bevy::prelude::*;

use crate::{hand::Hand, Field};

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
pub enum InputType {
    Hand(Hand),
    Tip(Isometry3d),
    Ray(Ray3d),
}

impl InputType {
    pub fn transform(self, mat: &Mat4) -> Self {
        match self {
            InputType::Hand(hand) => InputType::Hand(hand.transform(mat)),
            InputType::Tip(isometry) => InputType::Tip(Isometry3d {
                rotation: mat.to_scale_rotation_translation().1 * isometry.rotation,
                translation: mat.transform_point3a(isometry.translation),
            }),
            InputType::Ray(ray) => InputType::Ray(Ray3d {
                origin: mat.transform_point3(ray.origin),
                direction: Dir3::new_unchecked(mat.transform_vector3(ray.direction.as_vec3())),
            }),
        }
    }
    pub fn distance(&self, field: &Field, field_transform: &GlobalTransform) -> f32 {
        match self {
            InputType::Hand(hand) => hand.distance(field, field_transform),
            InputType::Tip(isometry) => field.distance(field_transform, isometry.translation),
            InputType::Ray(ray) => field.raymarch(field_transform, *ray).closest_distance,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InputData {
    pub input_method: Entity,
    pub input: InputType,
    pub minimal_non_spatial_data: NonSpatialInputData,
    pub handler_location: GlobalTransform,
    pub input_method_location: Isometry3d,
    pub distance: f32,
    pub captured: bool,
}
