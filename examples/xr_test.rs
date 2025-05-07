use bevy::prelude::*;
use bevy_mod_openxr::{add_xr_plugins, session::OxrSession};
use bevy_mod_xr::session::XrSessionCreated;
use bevy_suis::{
    debug::SuisDebugGizmosPlugin, xr::SuisXrPlugin, xr_controllers::SuisXrControllerPlugin,
    CaptureContext, Field, InputHandler, SuisCorePlugin,
};
use openxr::ReferenceSpaceType;

fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins((
            SuisCorePlugin,
            SuisXrPlugin,
            SuisDebugGizmosPlugin,
            SuisXrControllerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(XrSessionCreated, make_spectator_cam_follow)
        .run()
}

#[derive(Component)]
struct Cam;

fn make_spectator_cam_follow(
    query: Query<Entity, With<Cam>>,
    mut cmds: Commands,
    session: Res<OxrSession>,
) {
    let space = session
        .create_reference_space(ReferenceSpaceType::VIEW, Transform::IDENTITY)
        .unwrap();
    match query.single() {
        Ok(entity) => {
            cmds.entity(entity).insert(space.0);
        }
        Err(_) => {
            error!("No camera entity found, unable to attach space.");
            return;
        }
    }
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        InputHandler::new(capture_condition),
        Field::Sphere(0.2),
        Transform::from_translation(Vec3::new(0.0, 1.5, -1.0)),
        Visibility::default(),
    ));
    cmds.spawn((
        Camera3d::default(), 
        Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
        Visibility::default(),
        Cam,
    ));
}

fn capture_condition(ctx: In<CaptureContext>) -> bool {
    ctx.closest_point
        .distance(ctx.input_method_location.translation)
        <= f32::EPSILON
}
