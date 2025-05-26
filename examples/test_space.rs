use bevy::{color::palettes::css, prelude::*};
use bevy_suis::{debug::SuisDebugGizmosPlugin, field::Field, SuisCorePlugin};
fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((SuisCorePlugin, SuisDebugGizmosPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_pointer, draw_things).chain())
        .run()
}

#[derive(Component)]
#[require(Transform)]
pub struct Pointer;

fn setup(mut cmds: Commands) {
    cmds.spawn(Pointer);
    // cmds.spawn((Field::Sphere(0.5), Transform::from_xyz(0.0, -1.0, -5.)));
    // cmds.spawn((
    //     Field::Cuboid(Cuboid::new(1., 2., 3.)),
    //     Transform::from_xyz(0.0, -1.0, 0.0),
    // ));
    // cmds.spawn((
    //     Field::Torus(Torus::new(0.3, 0.5)),
    //     Transform::from_xyz(0.0, -1.0, 0.0),
    // ));
    cmds.spawn((
        Field::Cylinder(Cylinder::new(0.3, 0.5)),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
    cmds.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.5, 1.5, 3.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));
}

fn move_pointer(
    keys: Res<ButtonInput<KeyCode>>,
    mut pointer: Query<&mut Transform, With<Pointer>>,
    time: Res<Time>,
) {
    let mut p = pointer.single_mut().unwrap();
    if keys.pressed(KeyCode::KeyW) {
        p.translation.z += time.delta_secs() * -2.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        p.translation.z += time.delta_secs() * 2.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        p.translation.x += time.delta_secs() * -2.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        p.translation.x += time.delta_secs() * 2.0;
    }
    if keys.pressed(KeyCode::KeyE) {
        p.translation.y += time.delta_secs() * 2.0;
    }
    if keys.pressed(KeyCode::KeyQ) {
        p.translation.y += time.delta_secs() * -2.0;
    }
}

fn draw_things(
    mut giz: Gizmos,
    field_query: Query<(&GlobalTransform, &Field)>,
    pointer_query: Query<&GlobalTransform, With<Pointer>>,
) {
    let pointer = pointer_query.single().unwrap();
    for (f_pose, field) in &field_query {
        let pos = f_pose.translation();
        let closest_point = field.closest_point(f_pose, pointer.translation());
        let normal = field.normal(f_pose, pointer.translation());
        let distance = field.distance(f_pose, pointer.translation());
        giz.line(pos, pos + Vec3::from(normal), css::GOLD);
        giz.line(pos, pos + (Vec3::Y * distance), css::RED);
        giz.sphere(
            Isometry3d::from_translation(closest_point),
            0.01,
            css::MAGENTA,
        );
    }
    giz.axes(*pointer, 0.05);
}
