use bevy::{color::palettes::css, prelude::*};

use crate::{
    InputMethodDisabled, field::Field, input_method::InputMethod,
    input_method_data::SpatialInputData,
};
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
        &SpatialInputData,
        Has<InputMethodDisabled>,
    )>,
    mut gizmos: Gizmos,
) {
    for (transform, method, input, disabled) in &method_query {
        let color = match (!disabled, method.captured_by.is_some()) {
            (true, true) => css::LIME,
            (true, false) => css::BLUE,
            (false, _) => css::LIGHT_GRAY,
        };
        match input {
            // TODO: how to visualize hands without conflicting with bevy_mod_xr?
            SpatialInputData::Hand(_) => {}
            SpatialInputData::Tip(isometry) => {
                let t = Transform::from_isometry(*isometry);
                let base = t.with_translation(transform.transform_point(Vec3::Z * 0.02));

                gizmos.circle(base.to_isometry(), 0.01, color);
                gizmos.line(base.transform_point(Vec3::X * 0.01), t.translation, color);
                gizmos.line(base.transform_point(Vec3::X * -0.01), t.translation, color);
                gizmos.line(base.transform_point(Vec3::Y * 0.01), t.translation, color);
                gizmos.line(base.transform_point(Vec3::Y * -0.01), t.translation, color);
            }
            SpatialInputData::Ray(ray) => {
                gizmos.line(ray.origin, ray.origin + (*ray.direction * 0.2), color);
            }
        }
    }
}

fn draw_fields(field_query: Query<(&GlobalTransform, &Field)>, mut gizmos: Gizmos) {
    for (transform, field) in &field_query {
        match field {
            Field::Sphere(r) => {
                gizmos.sphere(transform.to_isometry(), *r, css::LIME);
            }
            Field::Cuboid(cuboid) => gizmos.cuboid(
                transform.mul_transform(Transform::from_scale(cuboid.half_size * 2.0)),
                css::LIME,
            ),
        }
    }
}
