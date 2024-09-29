use bevy::{color::palettes::css, prelude::*};
use bevy_suis::{debug::SuisDebugGizmosPlugin, Field, SuisCorePlugin};
fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((SuisCorePlugin, SuisDebugGizmosPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_pointer, draw_things).chain())
        .run()
}

#[derive(Component)]
pub struct Pointer;

fn setup(mut cmds: Commands) {
    cmds.spawn(Pointer).insert(SpatialBundle::default());
    cmds.spawn(Field::Sphere(0.5))
        .insert(SpatialBundle::default())
        .insert(Transform::from_xyz(0.0, -1.0, 0.0));
    cmds.spawn(Camera3dBundle::default())
        .insert(Transform::from_xyz(0.5, 1.5, 3.0).looking_at(Vec3::ZERO, Dir3::Y));
}

fn move_pointer(
    keys: Res<ButtonInput<KeyCode>>,
    mut pointer: Query<&mut Transform, With<Pointer>>,
    time: Res<Time>,
) {
    let mut p = pointer.single_mut();
    if keys.pressed(KeyCode::KeyW) {
        p.translation.z += time.delta_seconds() * -2.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        p.translation.z += time.delta_seconds() * 2.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        p.translation.x += time.delta_seconds() * -2.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        p.translation.x += time.delta_seconds() * 2.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        p.translation.y += time.delta_seconds() * 2.0;
    }
    if keys.pressed(KeyCode::KeyQ) {
        p.translation.y += time.delta_seconds() * -2.0;
    }
}

fn draw_things(
    mut giz: Gizmos,
    field_query: Query<(&GlobalTransform, &Field)>,
    pointer_query: Query<&GlobalTransform, With<Pointer>>,
) {
    let field = field_query.single();
    let pointer = pointer_query.single();
    let closest_point = field.1.closest_point2(field.0, pointer.translation());
    giz.axes(*pointer, 0.05);
    giz.sphere(closest_point, Quat::IDENTITY, 0.01, css::MAGENTA);
}
