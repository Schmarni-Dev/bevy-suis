pub mod default_bindings;
pub mod interaction_profiles;

use default_bindings::{
    SuisXrControllerActions, SuisXrControllerBindingSet, XrControllerInputActions,
};

use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{HandSide, LeftHand, RightHand},
    session::{XrPreDestroySession, XrSessionCreated, XrState, XrTrackingRoot},
    spaces::{XrSpaceLocationFlags, XrSpaceSyncSet},
};
use schminput::openxr::OxrInputPlugin;
use schminput::xr::AttachSpaceToEntity;
use schminput::{SchminputPlugin, SchminputSet, prelude::*};

use crate::{
    InputMethodDisabled, input_method::InputMethod, input_method_data::NonSpatialInputData,
    update_input_method_disabled,
};

pub struct SuisXrControllerPlugin;

impl Plugin for SuisXrControllerPlugin {
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
            (update_method_state, update_method_data)
                .after(SchminputSet::SyncInputActions)
                .after(XrSpaceSyncSet)
                .in_set(crate::SuisPreUpdateSets::UpdateInputMethods),
        );
    }
}

fn update_method_state(
    query: Query<
        (Entity, Has<InputMethodDisabled>, &XrSpaceLocationFlags),
        With<SuisXrControllerInputMethod>,
    >,
    mut cmds: Commands,
) {
    for (entity, disabled, flags) in &query {
        update_input_method_disabled(
            &mut cmds,
            entity,
            flags.position_tracked && flags.rotation_tracked,
            disabled,
        );
    }
}

fn update_method_data(
    vec2: Query<&Vec2ActionValue>,
    f32: Query<&F32ActionValue>,
    actions: Res<SuisXrControllerActions>,
    mut method_query: Query<
        (&mut NonSpatialInputData, &HandSide),
        (With<InputMethod>, With<SuisXrControllerInputMethod>),
    >,
    mut last_delta_scroll: Local<(Vec2, Vec2)>,
    time: Res<Time>,
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
    for (mut data, side) in &mut method_query {
        *data = match side {
            HandSide::Left => action_data_left,
            HandSide::Right => action_data_right,
        };
    }
}

#[derive(Default, Component)]
struct SuisXrControllerInputMethod;

fn setup(
    mut cmds: Commands,
    root: Query<Entity, With<XrTrackingRoot>>,
    action: Res<SuisXrControllerActions>,
) {
    let method_left = cmds
        .spawn((
            SuisXrControllerInputMethod,
            NonSpatialInputData::default(),
            HandSide::Left,
            LeftHand,
        ))
        .id();
    let method_right = cmds
        .spawn((
            SuisXrControllerInputMethod,
            NonSpatialInputData::default(),
            HandSide::Right,
            RightHand,
        ))
        .id();
    cmds.entity(action.actions_left.pose)
        .insert(AttachSpaceToEntity(method_left));
    cmds.entity(action.actions_right.pose)
        .insert(AttachSpaceToEntity(method_right));
    cmds.entity(root.single().unwrap())
        .add_child(method_left)
        .add_child(method_right);
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
