use bevy::{prelude::*, render::pipelined_rendering::PipelinedRenderingPlugin};
use bevy_mod_openxr::{add_xr_plugins, session::OxrSession};
use bevy_mod_xr::{
    camera::XrCamera,
    session::{session_running, XrSessionCreated},
};
use bevy_suis::{
    debug::SuisDebugGizmosPlugin,
    input_handler::InputHandler,
    input_method_data::NonSpatialInputData,
    window_pointers::SuisWindowPointerPlugin,
    xr::SuisXrPlugin,
    xr_controllers::{
        default_bindings::SuisXrControllerDefaultBindingsPlugin,
        interaction_profiles::SupportedInteractionProfiles, SuisXrControllerPlugin,
    },
    CaptureContext, Field, InputHandlerCaptures, PointerInputMethod, SuisCorePlugin,
};
use bevy_xr_utils::hand_gizmos::HandGizmosPlugin;
use openxr::ReferenceSpaceType;

// TODO: improve capturing mechanism
fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(
            DefaultPlugins.build().disable::<PipelinedRenderingPlugin>(),
        ))
        .add_plugins((
            SuisCorePlugin,
            SuisXrPlugin,
            SuisDebugGizmosPlugin,
            SuisXrControllerPlugin,
            SuisWindowPointerPlugin,
            SuisXrControllerDefaultBindingsPlugin {
                supported_interaction_profiles: SupportedInteractionProfiles::new()
                    .with_oculus_touch(),
            },
            HandGizmosPlugin,
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
            &InputHandlerCaptures,
            &GlobalTransform,
            &mut Transform,
            Option<&mut Grabbed>,
            Option<&Parent>,
        ),
        With<Grabble>,
    >,
    method_query: Query<(
        &GlobalTransform,
        &NonSpatialInputData,
        Has<PointerInputMethod>,
    )>,
    parent_query: Query<&GlobalTransform>,
    mut cmds: Commands,
) {
    for (handler_entity, handler, handler_gt, mut handler_transform, grabbed, parent) in
        &mut grabbles
    {
        let Some((method_transform, data, ranged)) = handler
            .captured_methods
            .first()
            .copied()
            .and_then(|v| method_query.get(v).ok())
        else {
            cmds.entity(handler_entity).remove::<Grabbed>();
            continue;
        };
        let grabbing = data.grab > 0.8;
        match (grabbed.is_some(), grabbing) {
            (false, true) => {
                cmds.entity(handler_entity)
                    .insert(Grabbed(Transform::from_matrix(
                        method_transform.compute_matrix().inverse() * handler_gt.compute_matrix(),
                    )));
            }
            (true, false) => {
                cmds.entity(handler_entity).remove::<Grabbed>();
            }
            _ => {}
        }
        if let Some(mut t) = grabbed {
            let w = parent
                .and_then(|v| parent_query.get(v.get()).ok())
                .copied()
                .unwrap_or(GlobalTransform::IDENTITY);
            if ranged {
                t.0.translation.z -= data.scroll.unwrap_or_default().y;
            }
            *handler_transform = Transform::from_matrix(
                method_transform.mul_transform(t.0).compute_matrix() * w.compute_matrix().inverse(),
            );
        }
    }
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
    cmds.entity(query.single()).insert(space.0);
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        InputHandler::new(),
        Field::Sphere(0.2),
        Transform::from_xyz(0.0, 0.5, -0.5),
        Grabble,
    ));
    cmds.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 110f32.to_radians(),
            ..Default::default()
        }),
        Cam,
    ));
}

fn capture_condition(
    ctx: In<CaptureContext>,
    query: Query<&NonSpatialInputData>,
    handler_query: Query<&InputHandlerCaptures>,
) -> bool {
    // Only Capture one method
    if !handler_query
        .get(ctx.handler)
        .is_ok_and(|v| v.captured_methods.is_empty())
    {
        return false;
    }

    let Ok(data) = query.get(ctx.input_method) else {
        return false;
    };
    if ctx.distance <= 0.01 {
        return data.grab > 0.8;
    }
    false
}
