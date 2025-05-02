use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{spawn_hand_bones, HandSide, SpawnHandTracker, XrHandBoneEntities, HAND_JOINT_COUNT},
    session::XrTracker,
    spaces::XrSpaceLocationFlags,
};

use crate::{
    hand::Hand,
    input_method::InputMethod,
    input_method_data::{NonSpatialInputData, SpatialInputData},
    SuisPreUpdateSets,
};
pub struct SuisDefaultXrHandsInputMethodPlugin;

impl Plugin for SuisDefaultXrHandsInputMethodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            update_hand_input_methods.in_set(SuisPreUpdateSets::UpdateInputMethods),
        );
    }
}

fn spawn_methods(mut cmds: Commands, joints: Res<CustomHandTrackerJoints>) {
    cmds.queue(SpawnHandTracker {
        joints: XrHandBoneEntities(joints.left),
        tracker_bundle: CustomHandTracker,
        side: HandSide::Left,
    });
    cmds.queue(SpawnHandTracker {
        joints: XrHandBoneEntities(joints.right),
        tracker_bundle: CustomHandTracker,
        side: HandSide::Right,
    });
    cmds.spawn((
        InputMethod::new(),
        SpatialInputData::Hand(Hand::empty()),
        SuisDefaultXrHandInputMethod,
        HandtrackingJoints(joints.left),
    ));
    cmds.spawn((
        InputMethod::new(),
        SpatialInputData::Hand(Hand::empty()),
        SuisDefaultXrHandInputMethod,
        HandtrackingJoints(joints.right),
    ));
}

#[derive(Clone, Copy, Component, Hash, Debug)]
struct HandtrackingJoints([Entity; HAND_JOINT_COUNT]);

#[derive(Clone, Copy, Component, Hash, Debug)]
pub struct SuisDefaultXrHandInputMethod;

#[derive(Resource)]
struct CustomHandTrackerJoints {
    left: [Entity; HAND_JOINT_COUNT],
    right: [Entity; HAND_JOINT_COUNT],
}

#[derive(Component)]
struct CustomHandTracker;

fn spawn_custom_handtracker_joints(mut cmds: Commands) {
    let left_joints = spawn_hand_bones(&mut cmds, |_| {
        (
            XrTracker,
            Transform::IDENTITY,
            XrSpaceLocationFlags::default(),
        )
    });
    let right_joints = spawn_hand_bones(&mut cmds, |_| {
        (
            XrTracker,
            Transform::IDENTITY,
            XrSpaceLocationFlags::default(),
        )
    });
    cmds.insert_resource(CustomHandTrackerJoints {
        left: left_joints,
        right: right_joints,
    });
}

fn update_hand_input_methods(
    mut query: Query<(
        &mut InputMethod,
        &mut SpatialInputData,
        &mut NonSpatialInputData,
    )>,
) {
    for (input_method, spatial_data, non_spatial_data) in &mut query {}
}
