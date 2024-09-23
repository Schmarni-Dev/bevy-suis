use bevy::prelude::*;
use bevy_mod_openxr::{add_xr_plugins, session::OxrSession};
use bevy_mod_xr::{
    camera::XrCamera,
    session::{session_running, XrSessionCreated},
};
use bevy_suis::{
    debug::SuisDebugGizmosPlugin,
    window_pointers::{MouseInputMethodData, SuisWindowPointerPlugin},
    xr::{Hand, HandInputMethodData, SuisXrPlugin},
    xr_controllers::{SuisXrControllerPlugin, XrControllerInputMethodData},
    CaptureContext, Field, InputHandler, PointerInputMethod, SuisCorePlugin,
};
use openxr::ReferenceSpaceType;

// TODO: improve capturing mechanism
fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins((
            SuisCorePlugin,
            SuisXrPlugin,
            SuisDebugGizmosPlugin,
            SuisXrControllerPlugin,
            SuisWindowPointerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(XrSessionCreated, make_spectator_cam_follow)
        .add_systems(Update, move_grabble)
        .add_systems(Update, update_camera.run_if(not(session_running)))
        .run()
}

fn update_camera(mut cams: Query<&mut Transform, (With<Camera>, Without<XrCamera>)>) {
    for mut transform in cams.iter_mut() {
        *transform = Transform::from_xyz(0.5, 2.0, 3.6).looking_at(Vec3::Y, Dir3::Y);
    }
}

#[derive(Clone, Copy, Component)]
struct Grabble;

#[derive(Clone, Copy, Component)]
struct Grabbed(Transform);

fn move_grabble(
    mut grabbles: Query<
        (
            Entity,
            &InputHandler,
            &GlobalTransform,
            &mut Transform,
            Option<&Grabbed>,
            Option<&Parent>,
        ),
        With<Grabble>,
    >,
    method_query: Query<(
        &GlobalTransform,
        Option<&HandInputMethodData>,
        Option<&XrControllerInputMethodData>,
        Option<&MouseInputMethodData>,
    )>,
    parent_query: Query<&GlobalTransform>,
    mut cmds: Commands,
) {
    for (handler_entity, handler, handler_gt, mut handler_transform, grabbed, parent) in
        &mut grabbles
    {
        let Some((method_transform, hand_data, controller_data, mouse_data)) = handler
            .captured_methods
            .first()
            .copied()
            .and_then(|v| method_query.get(v).ok())
        else {
            cmds.entity(handler_entity).remove::<Grabbed>();
            continue;
        };
        let mut grabbing = false;
        if let Some(hand) = hand_data {
            let hand = hand.get_in_relative_space(handler_gt);
            grabbing |= finger_separation(&hand, GRAB_SEPARATION);
        }
        if let Some(controller) = controller_data {
            grabbing |= controller.squeezed;
        }
        if let Some(mouse) = mouse_data {
            grabbing |= mouse.left_button.pressed;
        }
        match (grabbed, grabbing) {
            (None, true) => {
                cmds.entity(handler_entity)
                    .insert(Grabbed(Transform::from_matrix(
                        method_transform.compute_matrix().inverse() * handler_gt.compute_matrix(),
                    )));
            }
            (Some(_), false) => {
                cmds.entity(handler_entity).remove::<Grabbed>();
            }
            _ => {}
        }
        if let Some(t) = grabbed {
            let w = parent
                .and_then(|v| parent_query.get(v.get()).ok())
                .copied()
                .unwrap_or(GlobalTransform::IDENTITY);

            *handler_transform = Transform::from_matrix(
                method_transform.mul_transform(t.0).compute_matrix() * w.compute_matrix().inverse(),
            );
        }
    }
}

const GRAB_SEPARATION: f32 = 0.005;

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
    cmds.entity(query.single()).insert(space.0);
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        InputHandler::new(capture_condition),
        Field::Sphere(0.2),
        SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.5, -0.5)),
        Grabble,
    ));
    cmds.spawn((Camera3dBundle::default(), Cam));
}

fn capture_condition(
    ctx: In<CaptureContext>,
    query: Query<(
        Option<&HandInputMethodData>,
        Has<PointerInputMethod>,
        Option<&MouseInputMethodData>,
    )>,
    handler_query: Query<&InputHandler>,
) -> bool {
    // Only Capture one method
    if !handler_query
        .get(ctx.handler)
        .is_ok_and(|v| v.captured_methods.is_empty())
    {
        return false;
    }
    let method_distance = ctx
        .closest_point
        .distance(ctx.input_method_location.translation);

    // threshold needed to be this high else controllers wouldn't rellieably capture, idk why
    let mut capture = method_distance <= f32::EPSILON;
    let Ok((hand_data, is_pointer, mouse_data)) = query.get(ctx.input_method) else {
        return capture;
    };
    if let Some(hand_data) = hand_data {
        let hand = hand_data.get_in_relative_space(&ctx.handler_location);
        if method_distance < 0.1 {
            capture |= finger_separation(&hand, GRAB_SEPARATION * 1.5);
        }
    }
    if is_pointer {
        if let Some(mouse) = mouse_data {
            return mouse.left_button.pressed;
        }
        return true;
    }
    capture
}

fn finger_separation(hand: &Hand, max_seperation: f32) -> bool {
    hand.thumb.tip.pos.distance(hand.index.tip.pos)
        < hand.index.tip.radius + hand.thumb.tip.radius + max_seperation
}
