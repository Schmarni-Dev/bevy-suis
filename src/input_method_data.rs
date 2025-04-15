use bevy::prelude::*;

use crate::hand::HandInputMethodData;

#[derive(Clone, Copy, Component, Debug, Reflect, Default)]
pub struct InputMethodData {
    /// not available by defualt for hands
    pub scroll: Option<Vec2>,
    /// not available by defualt for hands
    pub pos: Option<Vec2>,
    pub hand: Option<HandInputMethodData>,
    pub select: f32,
    pub secondary: f32,
    pub context: f32,
    pub grab: f32,
}
