use bevy::{color::palettes::css, prelude::*};

use crate::{Field, InputMethod};
pub struct SuisDebugGizmosPlugin;

impl Plugin for SuisDebugGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_method_gizmos);
        app.add_systems(Update, draw_fields);
    }
}

fn draw_method_gizmos(method_query: Query<(&GlobalTransform, &InputMethod)>, mut gizmos: Gizmos) {
    for (transform, method) in &method_query {
        let color = match method.captured_by.is_some() {
            true => css::LIME,
            false => css::BLUE,
        };
        gizmos.cuboid(
            transform.compute_transform().with_scale(Vec3::splat(0.05)),
            color,
        );
    }
}

fn draw_fields(field_query: Query<(&GlobalTransform, &Field)>, mut gizmos: Gizmos) {
    for field in &field_query {
        let (_, field_rot, field_pos) = field.0.to_scale_rotation_translation();
        match field.1 {
            Field::Sphere(r) => {
                gizmos.sphere(field_pos, field_rot, *r, css::LIME);
            }
        }
    }
}
