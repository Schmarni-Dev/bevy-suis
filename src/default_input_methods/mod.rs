use bevy::app::{PluginGroup, PluginGroupBuilder};

#[cfg(feature = "xr")]
pub mod xr_controllers;
#[cfg(feature = "xr")]
pub mod xr_hands;

pub struct SuisBundledInputMethodPlugins;
impl PluginGroup for SuisBundledInputMethodPlugins {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>();
        #[cfg(feature = "xr")]
        let group = group.add(xr_hands::SuisBundledXrHandsInputMethodPlugin);
        #[cfg(feature = "xr")]
        let group = group.add(xr_controllers::SuisBundledXrControllerInputMethodPlugin);

        group
    }
}
