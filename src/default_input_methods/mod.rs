use bevy::app::{PluginGroup, PluginGroupBuilder};

pub mod window_pointer_rays;
#[cfg(feature = "xr")]
pub mod xr_controllers;
#[cfg(feature = "xr")]
pub mod xr_hands;

pub struct SuisBundledInputMethodPlugins;
impl PluginGroup for SuisBundledInputMethodPlugins {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>()
            .add(window_pointer_rays::SuisWindowPointerRayPlugin);
        #[cfg(feature = "xr")]
        let group = group
            .add(xr_hands::SuisBundledXrHandsInputMethodPlugin)
            .add(xr_controllers::SuisBundledXrControllerInputMethodPlugin);
        group
    }
}
