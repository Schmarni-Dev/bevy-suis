use bevy::prelude::*;
use bevy_mod_openxr::features::handtracking::OxrHandTracker;
use bevy_mod_xr::hands::{HandBone, HandBoneRadius, XrHandBoneEntities, HAND_JOINT_COUNT};

pub struct SUISXRPlugin;
impl Plugin for SUISXRPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, ());
    }
}

pub struct Joint {
    pos: Vec3,
    ori: Quat,
    radius: f32,
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
pub struct Finger {
    tip: Joint,
    distal: Joint,
    proximal: Joint,
    intermediate: Joint,
    metacarpal: Joint,
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
pub struct Thumb {
    tip: Joint,
    distal: Joint,
    proximal: Joint,
    metacarpal: Joint,
}
impl Thumb {
    fn set_joint(&mut self, bone: &HandBone, joint: Joint) {
        match bone {
            HandBone::ThumbMetacarpal => self.metacarpal = joint,
            HandBone::ThumbProximal => self.proximal = joint,
            HandBone::ThumbDistal => self.distal = joint,
            HandBone::ThumbTip => self.tip = joint,
            _ => (),
        }
    }
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
pub struct Hand {
    thumb: Thumb,
    index: Finger,
    middle: Finger,
    ring: Finger,
    little: Finger,
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
    const fn empty() -> Hand {
        Hand {
            thumb: Thumb::empty(),
            index: Finger::empty(),
            middle: Finger::empty(),
            ring: Finger::empty(),
            little: Finger::empty(),
        }
    }
}

fn get_hands(
    joint_query: Query<(&GlobalTransform, &HandBoneRadius)>,
    hand_query: Query<&XrHandBoneEntities>,
) {
    let hands = hand_query
        .iter()
        .filter_map(|v| joint_query.get_many(v.0).ok())
        .map(|v| Hand::from_data(&v))
        .collect::<Vec<_>>();
}

pub enum InputDataType {
    Hand(Hand),
    // Controller() // lol gotta do this one using the openxr action system :/
    Pointer(Ray3d), // needs datamap tho
}


// fn clear_handler_old_input<(mut handlers: Query<&mut InputHandler<T>>) {
//     for mut handler in handlers.iter_mut() {
//         handler.previous_frame_data = std::mem::take(&mut handler.current_data);
//     }
// }
