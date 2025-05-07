use bevy::prelude::*;
use schminput::{prelude::RequestedSubactionPaths, ActionSet};

use super::interaction_profiles::SupportedInteractionProfiles;

// This whole input scheme is horrible and really easy to break,
// if anyone has a better idea PLEASE TELL ME ABOUT IT
pub struct SuisXrControllerDefaultBindingsPlugin {
    pub supported_interaction_profiles: SupportedInteractionProfiles,
}
impl Plugin for SuisXrControllerDefaultBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SupportedProfiles(
            self.supported_interaction_profiles.clone(),
        ));
        app.add_systems(Startup, create_bindings.in_set(SuisXrControllerBindingSet));
    }
}

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct SuisXrControllerBindingSet;

#[derive(Resource, Deref)]
pub struct SupportedProfiles(SupportedInteractionProfiles);

// TODO: filter interaction_profiles based on extensions/openxr version
fn create_bindings(
    interaction_profiles: Res<SupportedProfiles>,
    mut cmds: Commands,
    // mut paths: ResMut<SubactionPaths>,
) {
    let set_left = cmds
        .spawn(ActionSet::new(
            "suis_xr_input_set_left",
            "Spatial Universal Interaction System XR Input Sources: Left Hand",
            u32::MAX,
        ))
        .id();
    let set_right = cmds
        .spawn(ActionSet::new(
            "suis_xr_input_set_right",
            "Spatial Universal Interaction System XR Input Sources: Right Hand",
            u32::MAX,
        ))
        .id();
    let req_paths = RequestedSubactionPaths::default();
    // req_paths.push(paths.get_or_create_path("/oxr/user/hand/left", &mut cmds));
    // req_paths.push(paths.get_or_create_path("/oxr/user/hand/right", &mut cmds));
    let actions_left =
        binding_gen::spawn_bindings_left(set_left, &mut cmds, &interaction_profiles, &req_paths);
    let actions_right =
        binding_gen::spawn_bindings_right(set_right, &mut cmds, &interaction_profiles, &req_paths);
    cmds.insert_resource(SuisXrControllerActions {
        set_left,
        set_right,
        actions_left,
        actions_right,
    });
}

#[derive(Clone, Copy, Debug, Resource)]
pub struct SuisXrControllerActions {
    pub set_left: Entity,
    pub set_right: Entity,
    pub actions_left: XrControllerInputActions,
    pub actions_right: XrControllerInputActions,
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::world::CommandQueue, platform::collections::HashSet};

    use super::*;

    #[test]
    fn check_for_non_init_actions() {
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmds = Commands::new(&mut queue, &world);

        let set_left = cmds
            .spawn(ActionSet::new(
                "suis_xr_input_set_left",
                "Spatial Universal Interaction System XR Input Sources: Left Hand",
                0,
            ))
            .id();
        let bindings_left = binding_gen::spawn_bindings_left(
            set_left,
            &mut cmds,
            &SupportedInteractionProfiles(HashSet::new()),
            &RequestedSubactionPaths::default(),
        );
        let set_right = cmds
            .spawn(ActionSet::new(
                "suis_xr_input_set_right",
                "Spatial Universal Interaction System XR Input Sources: Right Hand",
                0,
            ))
            .id();
        let bindings_right = binding_gen::spawn_bindings_right(
            set_right,
            &mut cmds,
            &SupportedInteractionProfiles(HashSet::new()),
            &RequestedSubactionPaths::default(),
        );
        fn check(bindings: XrControllerInputActions) -> bool {
            bindings
                .iter_fields()
                .enumerate()
                .flat_map(|(i, v)| {
                    let field_name = bindings.name_at(i).unwrap_or("NONE").to_string();
                    if v.try_downcast_ref::<Entity>().is_some() {
                        return vec![(field_name, v)];
                    }
                    println!("{}", v.reflect_type_path());
                    if let bevy::reflect::ReflectRef::Struct(s) = v.reflect_ref() {
                        s.iter_fields()
                            .enumerate()
                            .map(|(i, v)| {
                                let str =
                                    format!("{field_name}.{}", s.name_at(i).unwrap_or("NONE"));
                                (str, v)
                            })
                            .collect::<Vec<_>>()
                    } else {
                        vec![(field_name, v)]
                    }
                })
                .map(|(str, v)| {
                    println!("{str}");
                    v
                })
                .any(|v| {
                    v.try_downcast_ref::<Entity>()
                        .is_none_or(|v| *v == Entity::PLACEHOLDER)
                })
        }
        assert!(!check(bindings_left));
        assert!(!check(bindings_right));
    }
}

mod binding_gen {

    use super::*;
    use crate::{
        gen_bindings, xr_controllers::interaction_profiles::SupportedInteractionProfile as Profile,
    };
    use schminput::{
        openxr::OxrBindings, xr::SpaceActionValue as Space, Action, F32ActionValue as F32,
        Vec2ActionValue as Vec2,
    };

    use super::XrControllerInputActions;
    gen_bindings!(
        spawn_bindings_left,
        (
            scroll_continuous,
            "Scroll Continuous",
            Vec2,
            [
                Profile::Pico4 => ("/user/hand/left/input/thumbstick"),
                Profile::ViveCosmos => ("/user/hand/left/input/thumbstick"),
                Profile::ViveFocus3 => ("/user/hand/left/input/thumbstick"),
                Profile::OculusTouch => ("/user/hand/left/input/thumbstick"),
                Profile::ValveIndex => ("/user/hand/left/input/thumbstick"),
                Profile::HpReverbG2 => ("/user/hand/left/input/thumbstick")
            ]
        ),
        (
            scroll_delta,
            "Scroll Delta",
            Vec2,
            [
                Profile::ViveWand => ("/user/hand/left/input/trackpad"),
                Profile::ValveIndex => ("/user/hand/left/input/trackpad")
            ]
        ),
        (
            input_pos,
            "Scroll Input Position",
            Vec2,
            [
                Profile::Pico4 => ("/user/hand/left/input/thumbstick"),
                Profile::ViveWand => ("/user/hand/left/input/trackpad"),
                Profile::ViveCosmos => ("/user/hand/left/input/thumbstick"),
                Profile::ViveFocus3 => ("/user/hand/left/input/thumbstick"),
                Profile::OculusTouch => ("/user/hand/left/input/thumbstick"),
                // index has 2 bindings, thumbstick and trackpad, prefering thumbstick for now
                Profile::ValveIndex => ("/user/hand/left/input/thumbstick"),
                Profile::HpReverbG2 => ("/user/hand/left/input/thumbstick")
            ]
        ),
        (
            select,
            "Select",
            F32,
            [
                Profile::Pico4 => ("/user/hand/left/input/trigger/value"),
                Profile::ViveWand => ("/user/hand/left/input/trigger/value"),
                Profile::ViveCosmos => ("/user/hand/left/input/trigger/value"),
                Profile::ViveFocus3 => ("/user/hand/left/input/trigger/value"),
                Profile::OculusTouch => ("/user/hand/left/input/trigger/value"),
                Profile::ValveIndex => ("/user/hand/left/input/trigger/value"),
                Profile::HpReverbG2 => ("/user/hand/left/input/trigger/value")
            ]
        ),
        (
            grab,
            "Grab",
            F32,
            [
                Profile::Pico4 => ("/user/hand/left/input/squeeze/value"),
                Profile::ViveWand => ("/user/hand/left/input/squeeze/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/squeeze/value"),
                Profile::ViveCosmos => ("/user/hand/left/input/squeeze/click"),
                Profile::OculusTouch => ("/user/hand/left/input/squeeze/value"),
                Profile::ValveIndex => ("/user/hand/left/input/squeeze/value"),
                Profile::HpReverbG2 => ("/user/hand/left/input/squeeze/value")
            ]
        ),
        (
            secondary,
            "Secondary",
            F32,
            [
                Profile::Pico4 => ("/user/hand/left/input/y/click"),
                Profile::ViveCosmos => ("/user/hand/left/input/y/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/y/click"),
                Profile::OculusTouch => ("/user/hand/left/input/y/click"),
                Profile::ValveIndex => ("/user/hand/left/input/b/click"),
                Profile::HpReverbG2 => ("/user/hand/left/input/y/click")
            ]
        ),
        (
            context,
            "Context",
            F32,
            [
                Profile::Pico4 => ("/user/hand/left/input/x/click"),
                Profile::ViveCosmos => ("/user/hand/left/input/x/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/x/click"),
                Profile::OculusTouch => ("/user/hand/left/input/x/click"),
                Profile::ValveIndex => ("/user/hand/left/input/a/click"),
                Profile::HpReverbG2 => ("/user/hand/left/input/x/click")
            ]
        ),
        (
            pose,
            "Pose",
            Space,
            [
                Profile::ViveWand => ("/user/hand/left/input/aim/pose"),
                Profile::OculusTouch => ("/user/hand/left/input/aim/pose"),
                Profile::ValveIndex => ("/user/hand/left/input/aim/pose"),
                Profile::HpReverbG2 => ("/user/hand/left/input/aim/pose"),
                Profile::Pico4 => ("/user/hand/left/input/aim/pose"),
                Profile::ViveCosmos => ("/user/hand/left/input/aim/pose"),
                Profile::ViveFocus3 => ("/user/hand/left/input/aim/pose")
            ]
        )
    );
    gen_bindings!(
        spawn_bindings_right,
        (
            scroll_continuous,
            "Scroll Continuous",
            Vec2,
            [
                Profile::Pico4 => ("/user/hand/right/input/thumbstick"),
                Profile::ViveCosmos => ("/user/hand/right/input/thumbstick"),
                Profile::ViveFocus3 => ("/user/hand/right/input/thumbstick"),
                Profile::OculusTouch => ("/user/hand/right/input/thumbstick"),
                Profile::ValveIndex => ("/user/hand/right/input/thumbstick"),
                Profile::HpReverbG2 => ("/user/hand/right/input/thumbstick")
            ]
        ),
        (
            scroll_delta,
            "Scroll Delta",
            Vec2,
            [
                Profile::ViveWand => ("/user/hand/right/input/trackpad"),
                Profile::ValveIndex => ("/user/hand/right/input/trackpad")
            ]
        ),
        (
            input_pos,
            "Scroll Input Position",
            Vec2,
            [
                Profile::Pico4 => ("/user/hand/right/input/thumbstick"),
                Profile::ViveWand => ("/user/hand/right/input/trackpad"),
                Profile::ViveCosmos => ("/user/hand/right/input/thumbstick"),
                Profile::ViveFocus3 => ("/user/hand/right/input/thumbstick"),
                Profile::OculusTouch => ("/user/hand/right/input/thumbstick"),
                // index has 2 bindings, thumbstick and trackpad, prefering thumbstick for now
                Profile::ValveIndex => ("/user/hand/right/input/thumbstick"),
                Profile::HpReverbG2 => ("/user/hand/right/input/thumbstick")
            ]
        ),
        (
            select,
            "Select",
            F32,
            [
                Profile::Pico4 => ("/user/hand/right/input/trigger/value"),
                Profile::ViveWand => ("/user/hand/right/input/trigger/value"),
                Profile::ViveCosmos => ("/user/hand/right/input/trigger/value"),
                Profile::ViveFocus3 => ("/user/hand/right/input/trigger/value"),
                Profile::OculusTouch => ("/user/hand/right/input/trigger/value"),
                Profile::ValveIndex => ("/user/hand/right/input/trigger/value"),
                Profile::HpReverbG2 => ("/user/hand/right/input/trigger/value")
            ]
        ),
        (
            grab,
            "Grab",
            F32,
            [
                Profile::Pico4 => ("/user/hand/right/input/squeeze/value"),
                Profile::ViveWand => ("/user/hand/right/input/squeeze/click"),
                Profile::ViveFocus3 => ("/user/hand/right/input/squeeze/value"),
                Profile::ViveCosmos => ("/user/hand/right/input/squeeze/click"),
                Profile::OculusTouch => ("/user/hand/right/input/squeeze/value"),
                Profile::ValveIndex => ("/user/hand/right/input/squeeze/value"),
                Profile::HpReverbG2 => ("/user/hand/right/input/squeeze/value")
            ]
        ),
        (
            secondary,
            "Secondary",
            F32,
            [
                Profile::Pico4 => ("/user/hand/right/input/b/click"),
                Profile::ViveCosmos => ("/user/hand/right/input/b/click"),
                Profile::ViveFocus3 => ("/user/hand/right/input/b/click"),
                Profile::OculusTouch => ("/user/hand/right/input/b/click"),
                Profile::ValveIndex => ("/user/hand/right/input/b/click"),
                Profile::HpReverbG2 => ("/user/hand/right/input/b/click")
            ]
        ),
        (
            context,
            "Context",
            F32,
            [
                Profile::Pico4 => ("/user/hand/right/input/a/click"),
                Profile::ViveCosmos => ("/user/hand/right/input/a/click"),
                Profile::ViveFocus3 => ("/user/hand/right/input/a/click"),
                Profile::OculusTouch => ("/user/hand/right/input/a/click"),
                Profile::ValveIndex => ("/user/hand/right/input/a/click"),
                Profile::HpReverbG2 => ("/user/hand/right/input/a/click")
            ]
        ),
        (
            pose,
            "Pose",
            Space,
            [
                Profile::ViveWand => ("/user/hand/right/input/aim/pose"),
                Profile::OculusTouch => ("/user/hand/right/input/aim/pose"),
                Profile::ValveIndex => ("/user/hand/right/input/aim/pose"),
                Profile::HpReverbG2 => ("/user/hand/right/input/aim/pose"),
                Profile::Pico4 => ("/user/hand/right/input/aim/pose"),
                Profile::ViveCosmos => ("/user/hand/right/input/aim/pose"),
                Profile::ViveFocus3 => ("/user/hand/right/input/aim/pose")
            ]
        )
    );
}

#[macro_export]
macro_rules! gen_bindings {
    ($fn_name:ident,$(($($action:ident).+,$action_localized:literal, $type:ty, [$($profile:expr => ($($binding:literal),*)),*])),*) => {
        pub fn $fn_name(set: bevy::prelude::Entity, cmds: &mut bevy::prelude::Commands, profiles: &SupportedInteractionProfiles, paths: &RequestedSubactionPaths) -> XrControllerInputActions {
            let mut entities = make_placeholder_action_struct();
            $(
                let mut blueprint = OxrBindings::default();
                $(
                if profiles.contains(&$profile) {
                    blueprint = blueprint.interaction_profile($profile.get_path())
                    $( .binding($binding) )*
                    .end();
                }
                )*
                let action_name = stringify!($($action).+).replace(".", "_");
                entities.$($action).+ = cmds.spawn((
                    Action::new(action_name, $action_localized, set),
                    paths.clone(),
                    blueprint,
                    <$type>::default(),
                )).id();
            )*
            entities
        }
    };
}
fn make_placeholder_action_struct() -> XrControllerInputActions {
    XrControllerInputActions {
        scroll_continuous: Entity::PLACEHOLDER,
        scroll_delta: Entity::PLACEHOLDER,
        input_pos: Entity::PLACEHOLDER,
        select: Entity::PLACEHOLDER,
        secondary: Entity::PLACEHOLDER,
        context: Entity::PLACEHOLDER,
        grab: Entity::PLACEHOLDER,
        pose: Entity::PLACEHOLDER,
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct XrControllerInputActions {
    pub scroll_continuous: Entity,
    pub scroll_delta: Entity,
    pub input_pos: Entity,
    pub select: Entity,
    pub secondary: Entity,
    pub context: Entity,
    pub grab: Entity,
    pub pose: Entity,
}
