use bevy::prelude::*;
use std::hash::Hash;
pub mod debug;
pub mod default_input_methods;
pub mod field;
pub mod hand;
pub mod input_handler;
pub mod input_method;
pub mod input_method_capturing;
pub mod input_method_data;
pub mod window_pointers;
pub mod order_helper;

pub struct SuisCorePlugin;
impl Plugin for SuisCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_disabling_component::<InputMethodDisabled>();
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

#[derive(Debug, Clone, Copy, Component)]
pub struct InputMethodDisabled;

pub fn update_input_method_disabled(
    cmds: &mut Commands,
    entity: Entity,
    active: bool,
    currently_disabled: bool,
) {
    match (active, currently_disabled) {
        (true, true) => {
            cmds.entity(entity).remove::<InputMethodDisabled>();
        }
        (false, false) => {
            cmds.entity(entity).insert(InputMethodDisabled);
        }
        _ => {}
    }
}

#[derive(SystemSet, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum SuisPreUpdateSets {
    PrepareMethodEvents,
    UpdateInputMethods,
    CaptureInputMethods,
    SendInputData,
}
