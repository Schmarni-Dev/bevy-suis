use bevy::prelude::*;
use input_method::InputMethod;
use std::hash::Hash;
pub mod debug;
pub mod hand;
pub mod input_handler;
pub mod input_method;
pub mod input_method_capturing;
pub mod input_method_data;
pub mod window_pointers;
pub mod xr;
pub mod xr_controllers;
pub mod field;
pub mod default_input_methods;

pub struct SuisCorePlugin;
impl Plugin for SuisCorePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            (
                SuisPreUpdateSets::PrepareMethodEvents,
                SuisPreUpdateSets::UpdateInputMethods,
                SuisPreUpdateSets::CaptureInputMethods,
                SuisPreUpdateSets::SendInputData,
            ),
        );
    }
}

#[derive(Deref, DerefMut, Debug, Clone, Copy, Component)]
pub struct InputMethodActive(pub bool);

#[derive(SystemSet, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum SuisPreUpdateSets {
    PrepareMethodEvents,
    UpdateInputMethods,
    CaptureInputMethods,
    SendInputData,
}
