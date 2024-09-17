use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{LeftHand, RightHand},
    session::{XrPreDestroySession, XrSessionCreated, XrTrackingRoot},
};
use schminput::{
    openxr::{AttachSpaceToEntity, OxrInputPlugin},
    prelude::*,
    SchminputPlugin, SchminputSet,
};

use crate::{xr::HandSide, InputMethod};

pub struct SuisXrControllerPlugin;

impl Plugin for SuisXrControllerPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SchminputPlugin>() {
            // assuming that all plugins are missing, adding minimal plugins
            app.add_plugins(SchminputPlugin);
            app.add_plugins(OxrInputPlugin);
        }
        app.add_systems(XrSessionCreated, spawn_input_methods);
        app.add_systems(XrPreDestroySession, despawn_input_methods);
        app.add_systems(Startup, setup);
        app.add_systems(
            PreUpdate,
            update_method_data
                .after(SchminputSet::SyncInputActions)
                // probably not needed
                .before(crate::InputMethodCapturingSet),
        );
    }
}

fn update_method_data(
    bool_query: Query<&BoolActionValue>,
    mut method_query: Query<(&mut XrControllerInputMethodData, &HandSide), With<InputMethod>>,
    actions: Res<Actions>,
) {
    let trigger_pulled_left = bool_query
        .get(actions.trigger_pulled_left)
        .expect("not a bool action?");
    let trigger_pulled_right = bool_query
        .get(actions.trigger_pulled_right)
        .expect("not a bool action?");
    let squeezed_left = bool_query
        .get(actions.squeezed_left)
        .expect("not a bool action?");
    let squeezed_right = bool_query
        .get(actions.squeezed_right)
        .expect("not a bool action?");
    for (mut data, side) in &mut method_query {
        match side {
            HandSide::Left => {
                data.trigger_pulled = trigger_pulled_left.any;
                data.squeezed = squeezed_left.any;
            }
            HandSide::Right => {
                data.trigger_pulled = trigger_pulled_right.any;
                data.squeezed = squeezed_right.any;
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Resource)]
struct Actions {
    set: Entity,
    // this could be reduced with subaction paths, that would need another plugin from schminput
    trigger_pulled_left: Entity,
    trigger_pulled_right: Entity,
    squeezed_left: Entity,
    squeezed_right: Entity,
    method_pose_left: Entity,
    method_pose_right: Entity,
}

fn setup(mut cmds: Commands, root: Query<Entity, With<XrTrackingRoot>>) {
    let set = cmds
        .spawn(ActionSetBundle::new(
            "suis",
            "Spatial Universal Interaction System",
        ))
        .id();
    let trigger_pulled_left = cmds
        .spawn(ActionBundle::new(
            "trigger_pulled_left",
            "Left Trigger Pulled",
            set,
        ))
        .insert(BoolActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/trigger/value")
                .end(),
        )
        .id();
    let trigger_pulled_right = cmds
        .spawn(ActionBundle::new(
            "trigger_pulled_right",
            "Right Trigger Pulled",
            set,
        ))
        .insert(BoolActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/trigger/value")
                .end(),
        )
        .id();
    let squeezed_left = cmds
        .spawn(ActionBundle::new(
            "squeezed_left",
            "Left Grip Squeezed",
            set,
        ))
        .insert(BoolActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/squeeze/value")
                .end(),
        )
        .id();
    let squeezed_right = cmds
        .spawn(ActionBundle::new(
            "squeezed_right",
            "Right Grip Squeezed",
            set,
        ))
        .insert(BoolActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/squeeze/value")
                .end(),
        )
        .id();
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
    cmds.entity(root.single())
        .add_child(method_left)
        .add_child(method_right);
    let method_pose_left = cmds
        .spawn(ActionBundle::new(
            "method_pose_left",
            "Left Input Pose",
            set,
        ))
        .insert(SpaceActionValue::default())
        .insert(AttachSpaceToEntity(method_left))
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/aim/pose")
                .end(),
        )
        .id();
    let method_pose_right = cmds
        .spawn(ActionBundle::new(
            "method_pose_right",
            "Right Input Pose",
            set,
        ))
        .insert(SpaceActionValue::default())
        .insert(AttachSpaceToEntity(method_right))
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/aim/pose")
                .end(),
        )
        .id();

    cmds.insert_resource(Actions {
        set,
        trigger_pulled_left,
        trigger_pulled_right,
        squeezed_left,
        squeezed_right,
        method_pose_left,
        method_pose_right,
    });
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

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct XrControllerInputMethodData {
    pub trigger_pulled: bool,
    pub squeezed: bool,
    // pub trigger: Entity,
    // pub squeeze: Entity,
}
