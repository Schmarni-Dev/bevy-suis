use crate::InputMethodActive;
use bevy::{math::Affine3, prelude::*};
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
            &mut HandInputMethodData,
            &mut Transform,
            &SuisXrHandJoints,
            &mut InputMethodActive,
        ),
        With<InputMethod>,
    >,
    flag_query: Query<&XrSpaceLocationFlags>,
    xr_hand_joint_query: Query<(&GlobalTransform, &HandBoneRadius)>,
) {
    for (mut hand_data, mut method_transform, joint_entities, mut active) in &mut hand_method_query
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
        hand_data.set_in_global_space(hand);
    }
}

#[allow(clippy::type_complexity)]
fn despawn_input_hands(
    mut cmds: Commands,
    query: Query<Entity, Or<(With<SuisInputXrHand>, With<HandInputMethodData>)>>,
) {
    for e in &query {
        cmds.entity(e).despawn();
    }
}

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
        InputMethod::new(),
        HandInputMethodData::new(),
        SuisXrHandJoints(left_bones),
        LeftHand,
        HandSide::Left,
    ));
    cmds.spawn((
        SpatialBundle::default(),
        InputMethod::new(),
        HandInputMethodData::new(),
        SuisXrHandJoints(right_bones),
        RightHand,
        HandSide::Right,
    ));
}

#[derive(Component, Clone, Copy)]
pub struct HandInputMethodData(Hand);
impl HandInputMethodData {
    pub const fn new() -> HandInputMethodData {
        HandInputMethodData(Hand::empty())
    }

    pub fn set_in_global_space(&mut self, hand: Hand) {
        self.0 = hand;
    }

    pub fn get_in_relative_space(&self, relative_to: &GlobalTransform) -> Hand {
        let mat = &relative_to.compute_matrix().inverse();
        Hand {
            thumb: Thumb {
                tip: mul_joint(mat, self.0.thumb.tip),
                distal: mul_joint(mat, self.0.thumb.distal),
                proximal: mul_joint(mat, self.0.thumb.proximal),
                metacarpal: mul_joint(mat, self.0.thumb.metacarpal),
            },
            index: Finger {
                tip: mul_joint(mat, self.0.index.tip),
                distal: mul_joint(mat, self.0.index.distal),
                proximal: mul_joint(mat, self.0.index.proximal),
                intermediate: mul_joint(mat, self.0.index.intermediate),
                metacarpal: mul_joint(mat, self.0.index.metacarpal),
            },
            middle: Finger {
                tip: mul_joint(mat, self.0.middle.tip),
                distal: mul_joint(mat, self.0.middle.distal),
                proximal: mul_joint(mat, self.0.middle.proximal),
                intermediate: mul_joint(mat, self.0.middle.intermediate),
                metacarpal: mul_joint(mat, self.0.middle.metacarpal),
            },
            ring: Finger {
                tip: mul_joint(mat, self.0.ring.tip),
                distal: mul_joint(mat, self.0.ring.distal),
                proximal: mul_joint(mat, self.0.ring.proximal),
                intermediate: mul_joint(mat, self.0.ring.intermediate),
                metacarpal: mul_joint(mat, self.0.ring.metacarpal),
            },
            little: Finger {
                tip: mul_joint(mat, self.0.little.tip),
                distal: mul_joint(mat, self.0.little.distal),
                proximal: mul_joint(mat, self.0.little.proximal),
                intermediate: mul_joint(mat, self.0.little.intermediate),
                metacarpal: mul_joint(mat, self.0.little.metacarpal),
            },
        }
    }
}

fn mul_joint(mat: &Mat4, joint: Joint) -> Joint {
    Joint {
        pos: mat.transform_point(joint.pos),
        ori: mat.to_scale_rotation_translation().1 * joint.ori,
        radius: joint.radius,
    }
}

impl Default for HandInputMethodData {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Clone, Copy, Debug)]
pub struct Joint {
    pub pos: Vec3,
    pub ori: Quat,
    pub radius: f32,
}
impl Joint {
    const fn empty() -> Self {
        Self {
            pos: Vec3::ZERO,
            ori: Quat::IDENTITY,
            radius: 0.0,
        }
    }
    fn from_data((transform, radius): (&GlobalTransform, &HandBoneRadius)) -> Self {
        let (_, rot, pos) = transform.to_scale_rotation_translation();
        Self {
            pos,
            ori: rot,
            radius: radius.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Finger {
    pub tip: Joint,
    pub distal: Joint,
    pub proximal: Joint,
    pub intermediate: Joint,
    pub metacarpal: Joint,
}
impl Finger {
    const fn empty() -> Self {
        Self {
            tip: Joint::empty(),
            distal: Joint::empty(),
            proximal: Joint::empty(),
            intermediate: Joint::empty(),
            metacarpal: Joint::empty(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Thumb {
    pub tip: Joint,
    pub distal: Joint,
    pub proximal: Joint,
    pub metacarpal: Joint,
}
impl Thumb {
    const fn empty() -> Self {
        Self {
            tip: Joint::empty(),
            distal: Joint::empty(),
            proximal: Joint::empty(),
            metacarpal: Joint::empty(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Hand {
    pub thumb: Thumb,
    pub index: Finger,
    pub middle: Finger,
    pub ring: Finger,
    pub little: Finger,
}

impl Hand {
    pub fn pinch(&self) -> f32 {
        self.pinch_between(&self.thumb.tip, &self.index.tip)
    }

    pub fn grab(&self) -> f32 {
        self.pinch_between(&self.ring.tip, &self.ring.metacarpal)
    }

    pub fn pinch_between<'a>(&'a self, joint_1: &'a Joint, joint_2: &'a Joint) -> f32 {
        const PINCH_MAX: f32 = 0.11;
        const PINCH_ACTIVACTION_DISTANCE: f32 = 0.01;
        self.pinch_between_with_params(joint_1, joint_2, PINCH_ACTIVACTION_DISTANCE, PINCH_MAX)
    }
    pub fn pinch_between_with_params<'a>(
        &'a self,
        joint_1: &'a Joint,
        joint_2: &'a Joint,
        activation_distance: f32,
        pinch_max: f32,
    ) -> f32 {
        let combined_radius = joint_1.radius + joint_2.radius;
        let pinch_dist = joint_1.pos.distance(joint_2.pos) - combined_radius;
        (1.0 - ((pinch_dist - activation_distance) / (pinch_max - activation_distance)))
            .clamp(0.0, 1.0)
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
        }
    }
    pub const fn empty() -> Hand {
        Hand {
            thumb: Thumb::empty(),
            index: Finger::empty(),
            middle: Finger::empty(),
            ring: Finger::empty(),
            little: Finger::empty(),
        }
    }
}
