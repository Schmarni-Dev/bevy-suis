use bevy::{
    math::{Mat4, Quat, Vec3},
    prelude::{GlobalTransform, TransformPoint as _},
    reflect::Reflect,
};
#[cfg(feature = "xr")]
use bevy_mod_xr::hands::HandBone;

use crate::Field;

#[derive(Clone, Copy, Debug, Reflect)]
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
}
#[derive(Clone, Copy, Debug, Reflect)]
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
#[derive(Clone, Copy, Debug, Reflect)]
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
#[derive(Clone, Copy, Debug, Reflect)]
pub struct Hand {
    pub thumb: Thumb,
    pub index: Finger,
    pub middle: Finger,
    pub ring: Finger,
    pub little: Finger,
    pub palm: Joint,
    pub wrist: Joint,
}

impl Hand {
    pub fn pinch(&self, relative_to: &GlobalTransform) -> f32 {
        self.pinch_between(HandBone::ThumbTip, HandBone::IndexTip, relative_to)
    }

    pub fn grab(&self, relative_to: &GlobalTransform) -> f32 {
        self.pinch_between(HandBone::RingTip, HandJoint::RingMetacarpal, relative_to)
    }

    pub fn pinch_between(
        &self,
        joint_1: HandJoint,
        joint_2: HandJoint,
        relative_to: &GlobalTransform,
    ) -> f32 {
        const PINCH_MAX: f32 = 0.11;
        const PINCH_ACTIVACTION_DISTANCE: f32 = 0.01;
        self.pinch_between_with_params(
            joint_1,
            joint_2,
            PINCH_ACTIVACTION_DISTANCE,
            PINCH_MAX,
            relative_to,
        )
    }
    pub fn pinch_between_with_params(
        &self,
        joint_1: HandJoint,
        joint_2: HandJoint,
        activation_distance: f32,
        pinch_max: f32,
        relative_to: &GlobalTransform,
    ) -> f32 {
        let joint_1 = mul_joint(&relative_to.compute_matrix(), self.get_joint(joint_1));
        let joint_2 = mul_joint(&relative_to.compute_matrix(), self.get_joint(joint_2));
        let combined_radius = joint_1.radius + joint_2.radius;
        let pinch_dist = joint_1.pos.distance(joint_2.pos) - combined_radius;
        (1.0 - ((pinch_dist - activation_distance) / (pinch_max - activation_distance)))
            .clamp(0.0, 1.0)
    }
    pub const fn empty() -> Hand {
        Hand {
            thumb: Thumb::empty(),
            index: Finger::empty(),
            middle: Finger::empty(),
            ring: Finger::empty(),
            little: Finger::empty(),
            palm: Joint::empty(),
            wrist: Joint::empty(),
        }
    }
    pub fn transform(self, mat: &Mat4) -> Hand {
        Hand {
            thumb: Thumb {
                tip: mul_joint(mat, self.thumb.tip),
                distal: mul_joint(mat, self.thumb.distal),
                proximal: mul_joint(mat, self.thumb.proximal),
                metacarpal: mul_joint(mat, self.thumb.metacarpal),
            },
            index: Finger {
                tip: mul_joint(mat, self.index.tip),
                distal: mul_joint(mat, self.index.distal),
                proximal: mul_joint(mat, self.index.proximal),
                intermediate: mul_joint(mat, self.index.intermediate),
                metacarpal: mul_joint(mat, self.index.metacarpal),
            },
            middle: Finger {
                tip: mul_joint(mat, self.middle.tip),
                distal: mul_joint(mat, self.middle.distal),
                proximal: mul_joint(mat, self.middle.proximal),
                intermediate: mul_joint(mat, self.middle.intermediate),
                metacarpal: mul_joint(mat, self.middle.metacarpal),
            },
            ring: Finger {
                tip: mul_joint(mat, self.ring.tip),
                distal: mul_joint(mat, self.ring.distal),
                proximal: mul_joint(mat, self.ring.proximal),
                intermediate: mul_joint(mat, self.ring.intermediate),
                metacarpal: mul_joint(mat, self.ring.metacarpal),
            },
            little: Finger {
                tip: mul_joint(mat, self.little.tip),
                distal: mul_joint(mat, self.little.distal),
                proximal: mul_joint(mat, self.little.proximal),
                intermediate: mul_joint(mat, self.little.intermediate),
                metacarpal: mul_joint(mat, self.little.metacarpal),
            },
            palm: mul_joint(mat, self.palm),
            wrist: mul_joint(mat, self.wrist),
        }
    }
}

impl Hand {
    pub fn distance(&self, field: &Field, field_transform: &GlobalTransform) -> f32 {
        [
            self.thumb.tip,
            self.index.tip,
            self.middle.tip,
            self.ring.tip,
            self.little.tip,
        ]
        .map(|tip| field.distance(field_transform, tip.pos))
        .into_iter()
        .reduce(f32::min)
        .unwrap()
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
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
            palm: mul_joint(mat, self.0.palm),
            wrist: mul_joint(mat, self.0.wrist),
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

impl Hand {
    pub const fn get_joint(&self, joint: HandJoint) -> Joint {
        match joint {
            HandJoint::Palm => self.palm,
            HandJoint::Wrist => self.wrist,
            HandJoint::ThumbMetacarpal => self.thumb.metacarpal,
            HandJoint::ThumbProximal => self.thumb.proximal,
            HandJoint::ThumbDistal => self.thumb.distal,
            HandJoint::ThumbTip => self.thumb.tip,
            HandJoint::IndexMetacarpal => self.index.metacarpal,
            HandJoint::IndexProximal => self.index.proximal,
            HandJoint::IndexIntermediate => self.index.intermediate,
            HandJoint::IndexDistal => self.index.distal,
            HandJoint::IndexTip => self.index.tip,
            HandJoint::MiddleMetacarpal => self.middle.metacarpal,
            HandJoint::MiddleProximal => self.middle.proximal,
            HandJoint::MiddleIntermediate => self.middle.intermediate,
            HandJoint::MiddleDistal => self.middle.distal,
            HandJoint::MiddleTip => self.middle.tip,
            HandJoint::RingMetacarpal => self.ring.metacarpal,
            HandJoint::RingProximal => self.ring.proximal,
            HandJoint::RingIntermediate => self.ring.intermediate,
            HandJoint::RingDistal => self.ring.distal,
            HandJoint::RingTip => self.ring.tip,
            HandJoint::LittleMetacarpal => self.little.metacarpal,
            HandJoint::LittleProximal => self.little.proximal,
            HandJoint::LittleIntermediate => self.little.intermediate,
            HandJoint::LittleDistal => self.little.distal,
            HandJoint::LittleTip => self.little.tip,
        }
    }
}

#[cfg(not(feature = "xr"))]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum HandJoint {
    Palm = 0,
    Wrist = 1,
    ThumbMetacarpal = 2,
    ThumbProximal = 3,
    ThumbDistal = 4,
    ThumbTip = 5,
    IndexMetacarpal = 6,
    IndexProximal = 7,
    IndexIntermediate = 8,
    IndexDistal = 9,
    IndexTip = 10,
    MiddleMetacarpal = 11,
    MiddleProximal = 12,
    MiddleIntermediate = 13,
    MiddleDistal = 14,
    MiddleTip = 15,
    RingMetacarpal = 16,
    RingProximal = 17,
    RingIntermediate = 18,
    RingDistal = 19,
    RingTip = 20,
    LittleMetacarpal = 21,
    LittleProximal = 22,
    LittleIntermediate = 23,
    LittleDistal = 24,
    LittleTip = 25,
}
#[cfg(feature = "xr")]
pub type HandJoint = HandBone;
