use bevy::{platform::collections::HashSet, prelude::Deref};

// could i macro this and the get_path impl? yeah but i don't care enough rn
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SupportedInteractionProfile {
    /// `/interaction_profiles/htc/vive_controller`
    ViveWand,
    /// `/interaction_profiles/oculus/touch_controller`
    OculusTouch,
    /// `/interaction_profiles/valve/index_controller`
    ValveIndex,
    /// `/interaction_profiles/hp/mixed_reality_controller`
    HpReverbG2,
    /// `/interaction_profiles/bytedance/pico4_controller`
    /// Requires the `XR_BD_controller_interaction` extension on OpenXR 1.0
    Pico4,
    /// `/interaction_profiles/htc/vive_cosmos_controller`
    /// Requires the `XR_HTC_vive_cosmos_controller_interaction` extension on OpenXR 1.0
    ViveCosmos,
    /// `/interaction_profiles/htc/vive_focus3_controller`
    /// Requires the `XR_HTC_vive_focus3_controller_interaction` extension on OpenXR 1.0
    ViveFocus3,
}

impl SupportedInteractionProfile {
    pub const fn get_path(&self) -> &'static str {
        use SupportedInteractionProfile as Profile;
        match self {
            Profile::ViveWand => "/interaction_profiles/htc/vive_controller",
            Profile::OculusTouch => "/interaction_profiles/oculus/touch_controller",
            Profile::ValveIndex => "/interaction_profiles/valve/index_controller",
            Profile::HpReverbG2 => "/interaction_profiles/hp/mixed_reality_controller",
            Profile::Pico4 => "/interaction_profiles/bytedance/pico4_controller",
            Profile::ViveCosmos => "/interaction_profiles/htc/vive_cosmos_controller",
            Profile::ViveFocus3 => "/interaction_profiles/htc/vive_focus3_controller",
        }
    }
}

#[derive(Debug, Deref, Clone, Default)]
pub struct SupportedInteractionProfiles(pub HashSet<SupportedInteractionProfile>);

impl SupportedInteractionProfiles {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_profile(mut self, profile: SupportedInteractionProfile) -> Self {
        self.0.insert(profile);
        self
    }
    pub fn with_vive_wand(self) -> Self {
        self.with_profile(SupportedInteractionProfile::ViveWand)
    }
    pub fn with_valve_index(self) -> Self {
        self.with_profile(SupportedInteractionProfile::ValveIndex)
    }
    pub fn with_oculus_touch(self) -> Self {
        self.with_profile(SupportedInteractionProfile::OculusTouch)
    }
    pub fn with_reverb_g2(self) -> Self {
        self.with_profile(SupportedInteractionProfile::HpReverbG2)
    }
    pub fn with_vive_cosmos(self) -> Self {
        self.with_profile(SupportedInteractionProfile::ViveCosmos)
    }
    pub fn with_vive_focus3(self) -> Self {
        self.with_profile(SupportedInteractionProfile::ViveFocus3)
    }
    pub fn with_pico4(self) -> Self {
        self.with_profile(SupportedInteractionProfile::Pico4)
    }
}
