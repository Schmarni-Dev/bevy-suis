use bevy::{color::palettes::css, prelude::*};

use crate::{Field, InputMethod, InputMethodActive, PointerInputMethod};
pub struct SuisDebugGizmosPlugin;

impl Plugin for SuisDebugGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_method_gizmos);
        app.add_systems(Update, draw_fields);
    }
}

fn draw_method_gizmos(
    method_query: Query<(
        &GlobalTransform,
        &InputMethod,
        Option<&PointerInputMethod>,
        &InputMethodActive,
    )>,
    mut gizmos: Gizmos,
) {
    for (transform, method, pointer, active) in &method_query {
        let color = match (active.0, method.captured_by.is_some()) {
            (true, true) => css::LIME,
            (true, false) => css::BLUE,
            (false, _) => css::LIGHT_GRAY,
        };
        if let Some(ray) = pointer {
            gizmos.line(ray.0.origin, ray.0.origin + (*ray.0.direction * 0.2), color);
        } else {
            let t = transform.compute_transform();
            let base = t.with_translation(transform.transform_point(Vec3::Z * 0.02));
            gizmos.circle(base.translation, t.forward(), 0.01, color);
            gizmos.line(base.transform_point(Vec3::X * 0.01), t.translation, color);
            gizmos.line(base.transform_point(Vec3::X * -0.01), t.translation, color);
            gizmos.line(base.transform_point(Vec3::Y * 0.01), t.translation, color);
            gizmos.line(base.transform_point(Vec3::Y * -0.01), t.translation, color);
        }
    }
}

fn draw_fields(field_query: Query<(&GlobalTransform, &Field)>, mut gizmos: Gizmos) {
    for field in &field_query {
        let (_, field_rot, field_pos) = field.0.to_scale_rotation_translation();
        match field.1 {
            Field::Sphere(r) => {
                gizmos.sphere(field_pos, field_rot, *r, css::LIME);
            }
            Field::Cuboid(cuboid) => gizmos.cuboid(
                field
                    .0
                    .mul_transform(Transform::from_scale(cuboid.half_size * 2.0)),
                css::LIME,
            ),
        }
    }
}
