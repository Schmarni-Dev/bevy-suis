pub mod default_bindings;
pub mod interaction_profiles;

use std::cmp::Ordering;

use default_bindings::{
    SuisXrControllerActions, SuisXrControllerBindingSet, XrControllerInputActions,
};

use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{HandSide, LeftHand, RightHand},
    session::{XrPreDestroySession, XrSessionCreated, XrState},
    spaces::{XrSpaceLocationFlags, XrSpaceSyncSet},
};
use schminput::openxr::OxrInputPlugin;
use schminput::xr::AttachSpaceToEntity;
use schminput::{SchminputPlugin, SchminputSet, prelude::*};

use crate::{
    InputMethodDisabled,
    input_method::InputMethod,
    input_method_data::{NonSpatialInputData, SpatialInputData},
    order_helper::InputHandlerQueryHelper,
    update_input_method_disabled,
};

pub struct SuisBundledXrControllerInputMethodPlugin;

impl Plugin for SuisBundledXrControllerInputMethodPlugin {
    fn build(&self, app: &mut App) {
        if *app.world().resource::<XrState>() == XrState::Unavailable {
            return;
        }
        if !app.is_plugin_added::<SchminputPlugin>() {
            // assuming that all plugins are missing, adding minimal plugins
            app.add_plugins(SchminputPlugin);
            app.add_plugins(OxrInputPlugin);
        }
        app.add_systems(XrSessionCreated, spawn_input_methods);
        app.add_systems(XrPreDestroySession, despawn_input_methods);
        app.add_systems(Startup, setup.after(SuisXrControllerBindingSet));
        app.add_systems(
            PreUpdate,
            (
                update_method_state,
                update_method_data,
                update_handler_order,
            )
                .chain()
                .after(SchminputSet::SyncInputActions)
                .after(XrSpaceSyncSet)
                .in_set(crate::SuisPreUpdateSets::UpdateInputMethods),
        );
    }
}

fn update_handler_order(
    mut query: Query<(&mut InputMethod, &SpatialInputData), With<SuisXrControllerInputMethod>>,
    handler_query: InputHandlerQueryHelper,
) {
    for (mut method, spatial_data) in &mut query {
        let mut handlers =
            handler_query.query_all_handler_fields(|(handler, field, field_transform)| {
                (handler, spatial_data.distance(field, field_transform))
            });
        handlers.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        let handlers = handlers.into_iter().map(|(e, _)| e).collect();
        method.set_handler_order(handlers);
    }
}

fn update_method_state(
    query: Query<(Entity, Has<InputMethodDisabled>, &HandSide), With<SuisXrControllerInputMethod>>,
    left_pose: Query<&XrSpaceLocationFlags, (With<SuisXrControllerPoseSource>, With<LeftHand>)>,
    right_pose: Query<&XrSpaceLocationFlags, (With<SuisXrControllerPoseSource>, With<RightHand>)>,
    mut cmds: Commands,
) {
    let map = |f: &XrSpaceLocationFlags| f.position_tracked && f.rotation_tracked;
    let pose_left = left_pose.single().map(map).unwrap_or_default();
    let pose_right = right_pose.single().map(map).unwrap_or_default();
    for (entity, disabled, side) in &query {
        update_input_method_disabled(
            &mut cmds,
            entity,
            match side {
                HandSide::Left => pose_left,
                HandSide::Right => pose_right,
            },
            disabled,
        );
    }
}

fn update_method_data(
    vec2: Query<&Vec2ActionValue>,
    f32: Query<&F32ActionValue>,
    actions: Res<SuisXrControllerActions>,
    mut method_query: Query<
        (&mut NonSpatialInputData, &mut SpatialInputData, &HandSide),
        (With<InputMethod>, With<SuisXrControllerInputMethod>),
    >,
    mut last_delta_scroll: Local<(Vec2, Vec2)>,
    time: Res<Time>,
    left_pose: Query<&GlobalTransform, (With<SuisXrControllerPoseSource>, With<LeftHand>)>,
    right_pose: Query<&GlobalTransform, (With<SuisXrControllerPoseSource>, With<RightHand>)>,
) {
    fn get_data(
        vec2: &Query<&Vec2ActionValue>,
        f32: &Query<&F32ActionValue>,
        actions: &XrControllerInputActions,
        old_delta: &mut Vec2,
        time: &Time,
    ) -> NonSpatialInputData {
        NonSpatialInputData {
            scroll: Some(
                (vec2
                    .get(actions.scroll_continuous)
                    .map(|v| v.any)
                    .unwrap_or_default()
                    * time.delta_secs()
                    * 2.0)
                    + {
                        let delta = vec2
                            .get(actions.scroll_delta)
                            .map(|v| v.any)
                            .unwrap_or_default();
                        let out = delta - *old_delta;
                        *old_delta = delta;
                        out
                    },
            ),
            pos: Some(
                vec2.get(actions.scroll_continuous)
                    .map(|v| v.any)
                    .unwrap_or_default()
                    + vec2
                        .get(actions.scroll_delta)
                        .map(|v| v.any)
                        .unwrap_or_default(),
            ),
            select: f32.get(actions.select).map(|v| v.any).unwrap_or_default(),
            secondary: f32
                .get(actions.secondary)
                .map(|v| v.any)
                .unwrap_or_default(),
            context: f32.get(actions.context).map(|v| v.any).unwrap_or_default(),
            grab: f32.get(actions.grab).map(|v| v.any).unwrap_or_default(),
        }
    }
    let pose_left = left_pose
        .single()
        .map(|t| t.to_isometry())
        .unwrap_or_default();
    let pose_right = right_pose
        .single()
        .map(|t| t.to_isometry())
        .unwrap_or_default();
    let action_data_left = get_data(
        &vec2,
        &f32,
        &actions.actions_left,
        &mut last_delta_scroll.0,
        &time,
    );
    let action_data_right = get_data(
        &vec2,
        &f32,
        &actions.actions_right,
        &mut last_delta_scroll.1,
        &time,
    );
    for (mut non_spatial_data, mut spatial_data, side) in &mut method_query {
        *non_spatial_data = match side {
            HandSide::Left => action_data_left,
            HandSide::Right => action_data_right,
        };
        let pose = match side {
            HandSide::Left => pose_left,
            HandSide::Right => pose_right,
        };
        *spatial_data = match *spatial_data {
            SpatialInputData::Hand(_) | SpatialInputData::Tip(_) => SpatialInputData::Tip(pose),
            SpatialInputData::Ray(_) => SpatialInputData::Ray(Ray3d::new(
                pose.translation.into(),
                pose.rotation * Dir3::NEG_Z,
            )),
        }
    }
}

#[derive(Default, Component)]
struct SuisXrControllerInputMethod;
#[derive(Default, Component)]
struct SuisXrControllerPoseSource;

fn setup(mut cmds: Commands, action: Res<SuisXrControllerActions>) {
    cmds.spawn((
        InputMethod::default(),
        NonSpatialInputData::default(),
        SpatialInputData::Tip(Isometry3d::IDENTITY),
        SuisXrControllerInputMethod,
        HandSide::Left,
        LeftHand,
    ));
    cmds.spawn((
        InputMethod::default(),
        NonSpatialInputData::default(),
        SpatialInputData::Tip(Isometry3d::IDENTITY),
        SuisXrControllerInputMethod,
        HandSide::Right,
        RightHand,
    ));

    let left_pose = cmds.spawn((SuisXrControllerPoseSource, LeftHand)).id();
    let right_pose = cmds.spawn((SuisXrControllerPoseSource, RightHand)).id();

    cmds.entity(action.actions_left.pose)
        .insert(AttachSpaceToEntity(left_pose));
    cmds.entity(action.actions_right.pose)
        .insert(AttachSpaceToEntity(right_pose));
}

fn despawn_input_methods(
    mut cmds: Commands,
    query: Query<Entity, With<SuisXrControllerInputMethod>>,
) {
    for e in &query {
        cmds.entity(e).remove::<InputMethod>();
    }
}
fn spawn_input_methods(
    mut cmds: Commands,
    query: Query<Entity, With<SuisXrControllerInputMethod>>,
) {
    for e in &query {
        cmds.entity(e).insert(InputMethod::new());
    }
}
