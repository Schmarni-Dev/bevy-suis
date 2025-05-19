use std::cmp::Ordering;

use bevy::prelude::*;
use bevy_mod_xr::{
    hands::{
        HAND_JOINT_COUNT, HandBone, HandSide, SpawnHandTracker, XrHandBoneEntities,
        XrHandBoneRadius, spawn_hand_bones,
    },
    session::{XrSessionCreated, XrTracker},
    spaces::{XrSpaceLocationFlags, XrSpaceSyncSet},
};

use crate::{
    InputMethodDisabled, SuisPreUpdateSets,
    hand::{Finger, Hand, Joint, Thumb},
    input_method::InputMethod,
    input_method_data::{NonSpatialInputData, SpatialInputData},
    order_helper::InputHandlerQueryHelper,
    update_input_method_disabled,
};
pub struct SuisBundledXrHandsInputMethodPlugin;

impl Plugin for SuisBundledXrHandsInputMethodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (update_active, update_data)
                .in_set(SuisPreUpdateSets::UpdateInputMethods)
                .after(XrSpaceSyncSet),
        );
        app.add_systems(XrSessionCreated, spawn_methods);
        app.add_systems(Startup, spawn_custom_handtracker_joints);
    }
}

fn update_active(
    query: Query<
        (Entity, &HandtrackingJoints, Has<InputMethodDisabled>),
        With<SuisBundledXrHandInputMethod>,
    >,
    flag_query: Query<&XrSpaceLocationFlags>,
    mut cmds: Commands,
) {
    for (e, joints, disabled) in &query {
        if let Ok(flags) = flag_query.get_many(joints.0) {
            update_input_method_disabled(
                &mut cmds,
                e,
                flags
                    .iter()
                    .all(|f| f.position_tracked && f.rotation_tracked),
                disabled,
            );
        } else {
            warn!("unable to get joint location flags");
        }
    }
}

fn update_data(
    mut query: Query<
        (
            &mut InputMethod,
            &mut SpatialInputData,
            &mut NonSpatialInputData,
            &HandtrackingJoints,
        ),
        With<SuisBundledXrHandInputMethod>,
    >,
    joint_query: Query<(&GlobalTransform, &XrHandBoneRadius)>,
    handler_query: InputHandlerQueryHelper,
) {
    for (mut input_method, mut spatial_data, mut non_spatial_data, joints) in &mut query {
        let Ok(joint_data) = joint_query.get_many(joints.0) else {
            warn!("unable to get joints!");
            continue;
        };
        let hand = Hand::from_xr_data(&joint_data);
        non_spatial_data.select = hand.pinch(&GlobalTransform::IDENTITY);
        non_spatial_data.grab = hand.grab(&GlobalTransform::IDENTITY);
        non_spatial_data.secondary = hand.pinch_between(
            HandBone::ThumbTip,
            HandBone::MiddleTip,
            &GlobalTransform::IDENTITY,
        );
        non_spatial_data.context = hand.pinch_between(
            HandBone::ThumbTip,
            HandBone::RingTip,
            &GlobalTransform::IDENTITY,
        );
        *spatial_data = SpatialInputData::Hand(hand);

        let mut handlers =
            handler_query.query_all_handler_fields(|(handler, field, field_transform)| {
                (handler, spatial_data.distance(field, field_transform))
            });

        handlers.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        let handlers = handlers.into_iter().map(|(e, _)| e).collect();
        input_method.set_handler_order(handlers);
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
        SuisBundledXrHandInputMethod,
        HandtrackingJoints(joints.left),
    ));
    cmds.spawn((
        InputMethod::new(),
        SpatialInputData::Hand(Hand::empty()),
        SuisBundledXrHandInputMethod,
        HandtrackingJoints(joints.right),
    ));
}

#[derive(Clone, Copy, Component, Hash, Debug)]
struct HandtrackingJoints([Entity; HAND_JOINT_COUNT]);

#[derive(Clone, Copy, Component, Hash, Debug)]
pub struct SuisBundledXrHandInputMethod;

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

impl Hand {
    pub fn from_xr_data(data: &[(&GlobalTransform, &XrHandBoneRadius); HAND_JOINT_COUNT]) -> Hand {
        Hand {
            thumb: Thumb {
                tip: Joint::from_data(data[HandBone::ThumbTip as usize]),
                distal: Joint::from_data(data[HandBone::ThumbDistal as usize]),
                proximal: Joint::from_data(data[HandBone::ThumbProximal as usize]),
                metacarpal: Joint::from_data(data[HandBone::ThumbMetacarpal as usize]),
            },
            index: Finger {
                tip: Joint::from_data(data[HandBone::IndexTip as usize]),
                distal: Joint::from_data(data[HandBone::IndexDistal as usize]),
                proximal: Joint::from_data(data[HandBone::IndexProximal as usize]),
                intermediate: Joint::from_data(data[HandBone::IndexIntermediate as usize]),
                metacarpal: Joint::from_data(data[HandBone::IndexMetacarpal as usize]),
            },
            middle: Finger {
                tip: Joint::from_data(data[HandBone::MiddleTip as usize]),
                distal: Joint::from_data(data[HandBone::MiddleDistal as usize]),
                proximal: Joint::from_data(data[HandBone::MiddleProximal as usize]),
                intermediate: Joint::from_data(data[HandBone::MiddleIntermediate as usize]),
                metacarpal: Joint::from_data(data[HandBone::MiddleMetacarpal as usize]),
            },
            ring: Finger {
                tip: Joint::from_data(data[HandBone::RingTip as usize]),
                distal: Joint::from_data(data[HandBone::RingDistal as usize]),
                proximal: Joint::from_data(data[HandBone::RingProximal as usize]),
                intermediate: Joint::from_data(data[HandBone::RingIntermediate as usize]),
                metacarpal: Joint::from_data(data[HandBone::RingMetacarpal as usize]),
            },
            little: Finger {
                tip: Joint::from_data(data[HandBone::LittleTip as usize]),
                distal: Joint::from_data(data[HandBone::LittleDistal as usize]),
                proximal: Joint::from_data(data[HandBone::LittleProximal as usize]),
                intermediate: Joint::from_data(data[HandBone::LittleIntermediate as usize]),
                metacarpal: Joint::from_data(data[HandBone::LittleMetacarpal as usize]),
            },
            palm: Joint::from_data(data[HandBone::Palm as usize]),
            wrist: Joint::from_data(data[HandBone::Wrist as usize]),
        }
    }
}

impl Joint {
    fn from_data((transform, radius): (&GlobalTransform, &XrHandBoneRadius)) -> Self {
        let (_, rot, pos) = transform.to_scale_rotation_translation();
        Self {
            pos,
            rot,
            radius: radius.0,
        }
    }
}
