use bevy::{
    platform::collections::HashSet, prelude::*,
    render::pipelined_rendering::PipelinedRenderingPlugin,
};
use bevy_mod_openxr::{
    add_xr_plugins, reference_space::OxrReferenceSpacePlugin, session::OxrSession,
};
use bevy_mod_xr::{
    camera::XrCamera,
    hand_debug_gizmos::HandGizmosPlugin,
    session::{XrSessionCreated, session_running},
};
use bevy_suis::{
    SuisPlugins,
    debug::SuisDebugGizmosPlugin,
    default_input_methods::{
        SuisBundledInputMethodPlugins,
        xr_controllers::{
            default_bindings::SuisXrControllerDefaultBindingsPlugin,
            interaction_profiles::{SupportedInteractionProfile, SupportedInteractionProfiles},
        },
    },
    field::Field,
    handler_actions::single::SingleHandlerAction,
    input_handler::{FieldRef, InputHandler},
    input_method_data::SpatialInputData,
};
use openxr::ReferenceSpaceType;

fn main() -> AppExit {
    App::new()
        .add_plugins(
            add_xr_plugins(DefaultPlugins.build().disable::<PipelinedRenderingPlugin>()).set(
                OxrReferenceSpacePlugin {
                    default_primary_ref_space: ReferenceSpaceType::LOCAL,
                },
            ),
        )
        .add_plugins((
            SuisXrControllerDefaultBindingsPlugin {
                supported_interaction_profiles: SupportedInteractionProfiles(HashSet::from_iter([
                    SupportedInteractionProfile::OculusTouch,
                ])),
            },
            SuisPlugins,
            SuisDebugGizmosPlugin,
            HandGizmosPlugin,
            SuisBundledInputMethodPlugins,
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

#[derive(Clone, Copy, Component, Default)]
#[require(SingleHandlerAction)]
struct Grabbable(Option<Isometry3d>);

fn move_grabble(
    mut grabbles: Query<(
        &mut InputHandler,
        &mut Transform,
        &mut SingleHandlerAction,
        &mut Grabbable,
    )>,
) {
    for (mut handler, mut handler_transform, mut handler_action, mut grabbable) in &mut grabbles {
        handler_action.update(
            &mut handler,
            false,
            |data| data.distance <= 0.01,
            |data| data.non_spatial_data.grab > 0.8,
        );
        if handler_action.stopped_acting() {
            debug!("stopped");
            grabbable.0.take();
        }
        if let Some(actor) = handler_action
            .started_acting()
            .then(|| handler_action.actor(&handler))
            .flatten()
        {
            debug!("started");
            let iso = iso_from_spatial_data(actor.spatial_data);
            grabbable.0.replace(iso);
        }
        if let (Some(grab_actor_location), Some(actor)) =
            (grabbable.0.as_mut(), handler_action.actor(&handler))
        {
            let offset = match actor.spatial_data {
                SpatialInputData::Hand(_) | SpatialInputData::Tip(_) => Vec3A::ZERO,
                SpatialInputData::Ray(_) => {
                    Vec3A::NEG_Z * actor.non_spatial_data.scroll.unwrap_or_default().y
                }
            };
            let curr_actor_location = iso_from_spatial_data(actor.spatial_data);
            let actor_relative_to_parent = handler_transform.to_isometry() * curr_actor_location;
            let idk = actor_relative_to_parent
                * Isometry3d::from_translation(offset)
                * grab_actor_location.inverse();
            *handler_transform = Transform::from_isometry(idk).with_scale(handler_transform.scale);
        }
    }
}

fn iso_from_spatial_data(spatial_data: SpatialInputData) -> Isometry3d {
    match spatial_data {
        SpatialInputData::Hand(hand) => Isometry3d::new(hand.palm.pos, hand.palm.rot),
        SpatialInputData::Tip(iso) => iso,
        SpatialInputData::Ray(ray) => Transform::from_translation(ray.origin)
            .looking_to(ray.direction, Dir3::Y)
            .to_isometry(),
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
    cmds.entity(query.single().unwrap()).insert(space.0);
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        InputHandler::new(FieldRef::This),
        Field::Sphere(0.2),
        Transform::from_xyz(0.0, -1.0, -0.5),
        Grabbable::default(),
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
