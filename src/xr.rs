use crate::{
    hand::{Finger, Hand, Joint, Thumb},
    input_method_data::InputMethodData,
    InputMethodActive,
};
use bevy::prelude::*;
use bevy_mod_openxr::spaces::OxrSpaceLocationFlags;
use bevy_mod_xr::{
    hands::{HandBone, HandBoneRadius, LeftHand, RightHand, XrHandBoneEntities, HAND_JOINT_COUNT},
    session::{XrPreDestroySession, XrSessionCreated, XrTrackingRoot},
    spaces::XrSpaceLocationFlags,
};

use crate::{InputMethod, SuisPreUpdateSets};

pub struct SuisXrPlugin;
impl Plugin for SuisXrPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(XrSessionCreated, spawn_input_hands);
        app.add_systems(XrPreDestroySession, despawn_input_hands);
        app.add_systems(
            PreUpdate,
            update_hand_input_methods.in_set(SuisPreUpdateSets::UpdateInputMethods),
        );
    }
}
fn update_hand_input_methods(
    mut hand_method_query: Query<
        (
            &mut InputMethodData,
            &mut Transform,
            &SuisXrHandJoints,
            &mut InputMethodActive,
        ),
        (With<InputMethod>, With<HandInputMethod>),
    >,
    flag_query: Query<&XrSpaceLocationFlags>,
    xr_hand_joint_query: Query<(&GlobalTransform, &HandBoneRadius)>,
) {
    for (mut method_data, mut method_transform, joint_entities, mut active) in
        &mut hand_method_query
    {
        let Ok(joints) = xr_hand_joint_query.get_many(joint_entities.0) else {
            warn!("unable to get hand joints");
            continue;
        };
        let flags = flag_query
            .get(joint_entities.0[HandBone::IndexTip as usize])
            .copied()
            .unwrap_or_default();
        active.0 = flags.position_tracked || flags.rotation_tracked;

        let hand = Hand::from_data(&joints);
        *method_transform = joints[HandBone::IndexTip as usize].0.compute_transform();
        method_data.select = hand.pinch(&GlobalTransform::IDENTITY);
        method_data.grab = hand.grab(&GlobalTransform::IDENTITY);
        method_data.secondary = hand.pinch_between(
            HandBone::ThumbTip,
            HandBone::MiddleTip,
            &GlobalTransform::IDENTITY,
        );
        method_data.context = hand.pinch_between(
            HandBone::ThumbTip,
            HandBone::RingTip,
            &GlobalTransform::IDENTITY,
        );
        method_data
            .hand
            .get_or_insert_default()
            .set_in_global_space(hand);
    }
}

#[allow(clippy::type_complexity)]
fn despawn_input_hands(
    mut cmds: Commands,
    query: Query<Entity, Or<(With<SuisInputXrHand>, With<HandInputMethod>)>>,
) {
    for e in &query {
        cmds.entity(e).despawn();
    }
}
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct HandInputMethod;

fn spawn_input_hands(mut cmds: Commands, root: Query<Entity, With<XrTrackingRoot>>) {
    use bevy_mod_xr::hands::{spawn_hand_bones, HandSide};

    let Ok(root) = root.get_single() else {
        error!("unable to get tracking root, skipping hand creation");
        return;
    };
    let left_bones = spawn_hand_bones(&mut cmds, |_| {
        (
            SuisInputXrHand,
            LeftHand,
            HandSide::Left,
            XrSpaceLocationFlags::default(),
            OxrSpaceLocationFlags(openxr::SpaceLocationFlags::EMPTY),
        )
    });
    let right_bones = spawn_hand_bones(&mut cmds, |_| {
        (
            SuisInputXrHand,
            RightHand,
            HandSide::Right,
            XrSpaceLocationFlags::default(),
            OxrSpaceLocationFlags(openxr::SpaceLocationFlags::EMPTY),
        )
    });
    cmds.entity(root).push_children(&left_bones);
    cmds.entity(root).push_children(&right_bones);
    cmds.push(bevy_mod_xr::hands::SpawnHandTracker {
        joints: XrHandBoneEntities(left_bones),
        tracker_bundle: SuisInputXrHand,
        side: HandSide::Left,
    });
    cmds.push(bevy_mod_xr::hands::SpawnHandTracker {
        joints: XrHandBoneEntities(right_bones),
        tracker_bundle: SuisInputXrHand,
        side: HandSide::Right,
    });
    cmds.spawn((
        SpatialBundle::default(),
        InputMethodData::default(),
        InputMethod::new(),
        HandInputMethod,
        SuisXrHandJoints(left_bones),
        LeftHand,
        HandSide::Left,
    ));
    cmds.spawn((
        SpatialBundle::default(),
        InputMethod::new(),
        InputMethodData::default(),
        HandInputMethod,
        SuisXrHandJoints(right_bones),
        RightHand,
        HandSide::Right,
    ));
}

#[derive(Clone, Copy, Component, Debug)]
pub struct SuisXrHandJoints(pub [Entity; HAND_JOINT_COUNT]);

#[derive(Clone, Copy, Component, Debug)]
pub enum HandSide {
    Left,
    Right,
}
#[derive(Clone, Copy, Component, Debug)]
pub struct SuisInputXrHand;

impl Joint {
    fn from_data((transform, radius): (&GlobalTransform, &HandBoneRadius)) -> Self {
        let (_, rot, pos) = transform.to_scale_rotation_translation();
        Self {
            pos,
            ori: rot,
            radius: radius.0,
        }
    }
}

impl Hand {
    pub fn from_data(data: &[(&GlobalTransform, &HandBoneRadius); HAND_JOINT_COUNT]) -> Hand {
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
