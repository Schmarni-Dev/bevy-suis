use bevy::prelude::*;

#[derive(Clone, Copy, Component, Debug, Default, Reflect)]
pub struct XrControllerInputMethodData {
    pub trigger: Trigger,
    pub squeeze: Squeeze,
    pub stick: Stick,
    pub trackpad: Trackpad,
    // using north and south to add support for the leaked deckard controllers in the future
    pub button_north: TouchButton,
    pub button_south: TouchButton,

    pub thumbrest_touched: bool,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct Trigger {
    pub pull: f32,
    pub pulled: bool,
    pub touched: bool,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct Squeeze {
    pub value: f32,
    pub squeezed: bool,
    pub force: f32,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct Stick {
    pub pos: Vec2,
    pub touched: bool,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct Trackpad {
    pub pos: Vec2,
    pub pressed: bool,
    pub touched: bool,
    pub force: f32,
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct TouchButton {
    pub pressed: bool,
    pub touched: bool,
}
