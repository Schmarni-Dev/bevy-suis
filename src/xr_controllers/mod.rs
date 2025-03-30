pub mod default_bindings;
pub mod input_method_data;
pub mod interaction_profiles;
mod query_ext;

use default_bindings::{SuisXrActions, SuisXrControllerBindingSet, XrControllerInputActions};
pub use input_method_data::XrControllerInputMethodData;
use input_method_data::{Squeeze, Stick, TouchButton, Trackpad, Trigger};

use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{LeftHand, RightHand},
    session::{XrPreDestroySession, XrSessionCreated, XrState, XrTrackingRoot},
    spaces::XrSpaceLocationFlags,
};
use query_ext::ActionValueQueryExt;
use schminput::{openxr::OxrInputPlugin, subaction_paths::SubactionPathPlugin};
use schminput::{prelude::*, SchminputPlugin, SchminputSet};
use schminput::{subaction_paths::SubactionPath, xr::AttachSpaceToEntity};

use crate::InputMethodActive;

use crate::{xr::HandSide, InputMethod};

pub struct SuisXrControllerPlugin;

impl Plugin for SuisXrControllerPlugin {
    fn build(&self, app: &mut App) {
        if *app.world().resource::<XrState>() == XrState::Unavailable {
            return;
        }
        if !app.is_plugin_added::<SchminputPlugin>() {
            // assuming that all plugins are missing, adding minimal plugins
            app.add_plugins(SchminputPlugin);
            app.add_plugins(SubactionPathPlugin);
            app.add_plugins(OxrInputPlugin);
        }
        app.add_systems(XrSessionCreated, spawn_input_methods);
        app.add_systems(XrPreDestroySession, despawn_input_methods);
        app.add_systems(Startup, setup.after(SuisXrControllerBindingSet));
        #[cfg(not(target_family = "wasm"))]
        {
            use bevy_mod_openxr::spaces::OxrSpaceSyncSet;
            app.add_systems(
                PreUpdate,
                (update_method_state, update_method_data)
                    .after(SchminputSet::SyncInputActions)
                    .after(OxrSpaceSyncSet)
                    .in_set(crate::SuisPreUpdateSets::UpdateInputMethods),
            );
        }
        // Not Perfect since we don't schedule against SpaceSync, will be fixed in the next
        // bevy_mod_xr version
        #[cfg(target_family = "wasm")]
        {
            app.add_systems(
                PreUpdate,
                (update_method_state, update_method_data)
                    .after(SchminputSet::SyncInputActions)
                    .in_set(crate::SuisPreUpdateSets::UpdateInputMethods),
            );
        }
    }
}

fn update_method_state(mut query: Query<(&mut InputMethodActive, &XrSpaceLocationFlags)>) {
    for (mut active, flags) in &mut query {
        active.0 = flags.position_tracked || flags.rotation_tracked;
    }
}

fn update_method_data(
    vec2: Query<&Vec2ActionValue>,
    f32: Query<&F32ActionValue>,
    bool: Query<&BoolActionValue>,
    actions: Res<XrControllerInputActions>,
    mut method_query: Query<(&mut XrControllerInputMethodData, &HandSide), With<InputMethod>>,
    mut paths: ResMut<SubactionPaths>,
    mut cmds: Commands,
) {
    fn get_data(
        vec2: &Query<&Vec2ActionValue>,
        f32: &Query<&F32ActionValue>,
        bool: &Query<&BoolActionValue>,
        actions: &XrControllerInputActions,
        path: &SubactionPath,
    ) -> XrControllerInputMethodData {
        XrControllerInputMethodData {
            trigger: Trigger {
                pull: f32.get_with_path_or_default(actions.trigger.pull, path),
                pulled: bool.get_with_path_or_default(actions.trigger.pulled, path),
                touched: bool.get_with_path_or_default(actions.trigger.touched, path),
            },
            squeeze: Squeeze {
                value: f32.get_with_path_or_default(actions.squeeze.value, path),
                squeezed: bool.get_with_path_or_default(actions.squeeze.squeezed, path),
                force: f32.get_with_path_or_default(actions.squeeze.force, path),
            },
            stick: Stick {
                pos: vec2.get_with_path_or_default(actions.stick.pos, path),
                touched: bool.get_with_path_or_default(actions.stick.touched, path),
            },
            trackpad: Trackpad {
                pos: vec2.get_with_path_or_default(actions.trackpad.pos, path),
                pressed: bool.get_with_path_or_default(actions.trackpad.pressed, path),
                touched: bool.get_with_path_or_default(actions.trackpad.touched, path),
                force: f32.get_with_path_or_default(actions.trackpad.force, path),
            },
            button_north: TouchButton {
                pressed: bool.get_with_path_or_default(actions.button_north.pressed, path),
                touched: bool.get_with_path_or_default(actions.button_north.touched, path),
            },
            button_south: TouchButton {
                pressed: bool.get_with_path_or_default(actions.button_south.pressed, path),
                touched: bool.get_with_path_or_default(actions.button_south.touched, path),
            },
            thumbrest_touched: bool.get_with_path_or_default(actions.thumbrest_touched, path),
        }
    }
    let action_data_left = get_data(
        &vec2,
        &f32,
        &bool,
        &actions,
        &paths.get_or_create_path("/oxr/user/hand/left", &mut cmds),
    );
    let action_data_right = get_data(
        &vec2,
        &f32,
        &bool,
        &actions,
        &paths.get_or_create_path("/oxr/user/hand/right", &mut cmds),
    );
    for (mut data, side) in &mut method_query {
        *data = match side {
            HandSide::Left => action_data_left,
            HandSide::Right => action_data_right,
        };
    }
}

fn setup(
    mut cmds: Commands,
    root: Query<Entity, With<XrTrackingRoot>>,
    action: Res<SuisXrActions>,
) {
    let method_left = cmds
        .spawn((
            XrControllerInputMethodData::default(),
            SpatialBundle::default(),
            HandSide::Left,
            LeftHand,
        ))
        .id();
    let method_right = cmds
        .spawn((
            XrControllerInputMethodData::default(),
            SpatialBundle::default(),
            HandSide::Right,
            RightHand,
        ))
        .id();
    cmds.entity(action.space_left)
        .insert(AttachSpaceToEntity(method_left));
    cmds.entity(action.space_right)
        .insert(AttachSpaceToEntity(method_right));
    cmds.entity(root.single())
        .add_child(method_left)
        .add_child(method_right);
}

fn despawn_input_methods(
    mut cmds: Commands,
    query: Query<Entity, With<XrControllerInputMethodData>>,
) {
    for e in &query {
        cmds.entity(e).remove::<InputMethod>();
    }
}
fn spawn_input_methods(
    mut cmds: Commands,
    query: Query<Entity, With<XrControllerInputMethodData>>,
) {
    for e in &query {
        cmds.entity(e).insert(InputMethod::new());
    }
}
