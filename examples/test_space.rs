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
    cmds.spawn(Pointer).insert((
        Transform::default(),
        Visibility::default(),
    ));
    cmds.spawn(Field::Sphere(0.5))
        .insert(Visibility::default())
        .insert(Transform::from_xyz(0.0, -1.0, -5.));
    cmds.spawn(Field::Cuboid(Cuboid::new(1., 2., 3.)))
        .insert(Visibility::default())
        .insert(Transform::from_xyz(0.0, -1.0, 0.0));
    cmds.spawn(Camera3d::default())
        .insert(Visibility::default())
        .insert(Transform::from_xyz(0.5, 1.5, 3.0).looking_at(Vec3::ZERO, Dir3::Y));
}

fn move_pointer(
    keys: Res<ButtonInput<KeyCode>>,
    mut pointer: Query<&mut Transform, With<Pointer>>,
    time: Res<Time>,
) {
    match pointer.single_mut() {
        Ok(mut p) => {
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
        Err(_) => {
            error!("No pointer found!");
        }
    }
}

fn draw_things(
    mut giz: Gizmos,
    field_query: Query<(&GlobalTransform, &Field)>,
    pointer_query: Query<&GlobalTransform, With<Pointer>>,
) {
    let pointer = pointer_query.single();

    match pointer {
        Ok(p) => {
            for (f_pose, field) in &field_query {
                let pos = f_pose.translation();
                let closest_point = field.closest_point(f_pose, p.translation());
                let normal = field.normal(f_pose, p.translation());
                let distance = field.distance(f_pose, p.translation());
                giz.line(pos, pos + normal, css::GOLD);
                giz.line(pos, pos + (Vec3::Y * distance), css::RED);
                giz.sphere(Isometry3d::new(closest_point, Quat::IDENTITY), 0.01, css::MAGENTA);
            }
            giz.axes(*p, 0.05);
        }, 
        Err(_) => {
            error!("No pointer found!");
            return;
        }
    }
    
}
