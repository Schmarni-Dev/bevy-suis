use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{LeftHand, RightHand},
    session::{XrPreDestroySession, XrSessionCreated, XrState, XrTrackingRoot},
    spaces::XrSpaceLocationFlags,
};
use schminput::openxr::OxrInputPlugin;
use schminput::xr::AttachSpaceToEntity;
use schminput::{prelude::*, SchminputPlugin, SchminputSet};

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
            app.add_plugins(OxrInputPlugin);
        }
        app.add_systems(XrSessionCreated, spawn_input_methods);
        app.add_systems(XrPreDestroySession, despawn_input_methods);
        app.add_systems(Startup, setup);
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
    vec2_query: Query<&Vec2ActionValue>,
    f32_query: Query<&F32ActionValue>,
    mut method_query: Query<
        (&ControllerActions, &mut XrControllerInputMethodData),
        With<InputMethod>,
    >,
) {
    for (actions, mut data) in &mut method_query {
        let trigger_pulled = f32_query
            .get(actions.trigger_pulled)
            .expect("not an f32 action?");
        let squeezed = f32_query.get(actions.squeezed).expect("not an f32 action?");
        let stick_pos = vec2_query
            .get(actions.stick_pos)
            .expect("not a Vec2 action?");
        let secondary_interact_data = f32_query
            .get(actions.secondary_interact)
            .expect("not an f32 action?");
        data.trigger_pull = trigger_pulled.any;
        data.squeeze = squeezed.any;
        data.stick_pos = stick_pos.any;
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Resource)]
pub struct Actions {
    pub set: Entity,
    // this could be reduced with subaction paths, that would need another plugin from schminput
    pub trigger_pulled_left: Entity,
    pub trigger_pulled_right: Entity,
    pub squeezed_left: Entity,
    pub squeezed_right: Entity,
    pub stick_pos_left: Entity,
    pub stick_pos_right: Entity,
    pub method_pose_left: Entity,
    pub method_pose_right: Entity,
    pub secondary_interact_left: Entity,
    pub secondary_interact_right: Entity,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Component)]
struct ControllerActions {
    trigger_pulled: Entity,
    squeezed: Entity,
    stick_pos: Entity,
    method_pose: Entity,
    secondary_interact: Entity,
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
        .insert(F32ActionValue::default())
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
        .insert(F32ActionValue::default())
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
        .insert(F32ActionValue::default())
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
        .insert(F32ActionValue::default())
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
    let stick_pos_left = cmds
        .spawn(ActionBundle::new(
            "stick_value_left",
            "Left Stick Position",
            set,
        ))
        .insert(Vec2ActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/thumbstick")
                .end(),
        )
        .id();

    let stick_pos_right = cmds
        .spawn(ActionBundle::new(
            "stick_value_right",
            "Right Stick Position",
            set,
        ))
        .insert(Vec2ActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/thumbstick")
                .end(),
        )
        .id();

    let secondary_interact_left = cmds
        .spawn(ActionBundle::new(
            "secondary_interact_left",
            "Left Secondary Interact Position",
            set,
        ))
        .insert(F32ActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/left/input/y/click")
                .end(),
        )
        .id();
    let secondary_interact_right = cmds
        .spawn(ActionBundle::new(
            "secondary_interact_right",
            "Right Secondary Interact Position",
            set,
        ))
        .insert(F32ActionValue::default())
        // TODO: add more bindings
        .insert(
            OxrActionBlueprint::default()
                .interaction_profile(OCULUS_TOUCH_PROFILE)
                .binding("/user/hand/right/input/b/click")
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
        stick_pos_left,
        stick_pos_right,
        secondary_interact_left,
        secondary_interact_right,
    });
    cmds.entity(method_left).insert(ControllerActions {
        trigger_pulled: trigger_pulled_left,
        squeezed: squeezed_left,
        stick_pos: stick_pos_left,
        method_pose: method_pose_left,
        secondary_interact: secondary_interact_left,
    });
    cmds.entity(method_right).insert(ControllerActions {
        trigger_pulled: trigger_pulled_right,
        squeezed: squeezed_right,
        stick_pos: stick_pos_right,
        method_pose: method_pose_right,
        secondary_interact: secondary_interact_right,
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
    pub select: bool,
    pub trigger_pull: f32,
    pub trigger_pulled: bool,
    pub squeeze: f32,
    pub squeezed: bool,
    pub stick_pos: Vec2,
    pub stick_touched: bool,
    pub touchpad_pos: Vec2,
    pub touchpad_pressed: bool,
    pub touchpad_touched: bool,
    pub secondary_interact: f32,
}
