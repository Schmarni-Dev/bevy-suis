use bevy::prelude::*;

/// value is between 0 and 1
#[derive(Deref, DerefMut, Default, Debug, Clone, Copy, Component)]
pub struct StrongGrab(pub f32);
/// value is between 0 and 1
#[derive(Deref, DerefMut, Default, Debug, Clone, Copy, Component)]
pub struct LightGrab(pub f32);
/// value is between 0 and 1
#[derive(Deref, DerefMut, Default, Debug, Clone, Copy, Component)]
pub struct PrimaryInteract(pub f32);
/// value is between 0 and 1
#[derive(Deref, DerefMut, Default, Debug, Clone, Copy, Component)]
pub struct SecondaryInteract(pub f32);
/// both the x and y axis is between -1 and 1
#[derive(Deref, DerefMut, Default, Debug, Clone, Copy, Component)]
pub struct Scroll(pub Vec2);
