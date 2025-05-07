use bevy::{prelude::*, render::pipelined_rendering::PipelinedRenderingPlugin};
use bevy_mod_openxr::{add_xr_plugins, session::OxrSession};
use bevy_mod_xr::{
    camera::XrCamera, hand_debug_gizmos::HandGizmosPlugin, session::{session_running, XrSessionCreated}
};
use bevy_suis::{
    debug::SuisDebugGizmosPlugin,
    input_method_data::InputMethodData,
    window_pointers::SuisWindowPointerPlugin,
    xr::SuisXrPlugin,
    xr_controllers::{
        default_bindings::SuisXrControllerDefaultBindingsPlugin,
        interaction_profiles::SupportedInteractionProfiles, SuisXrControllerPlugin,
    },
    CaptureContext, Field, InputHandler, InputHandlerCaptures, PointerInputMethod, SuisCorePlugin,
};
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
            Option<&ChildOf>,
        ),
        With<Grabble>,
    >,
    method_query: Query<(&GlobalTransform, &InputMethodData, Has<PointerInputMethod>)>,
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
                .and_then(|v| parent_query.get(v.parent()).ok())
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
        Transform::from_xyz(0.0, 0.5, -0.5),
        Visibility::default(),
        Grabble,
    ));
    cmds.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 110f32.to_radians(),
            ..Default::default()
        }),
        Transform::default(),
        Cam,
    ));
}

fn capture_condition(
    ctx: In<CaptureContext>,
    query: Query<&InputMethodData>,
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
