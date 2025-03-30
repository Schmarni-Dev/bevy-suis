use bevy::prelude::*;
use schminput::{
    openxr::OxrActionBlueprint,
    prelude::{RequestedSubactionPaths, SubactionPaths},
    xr::SpaceActionValue,
    ActionBundle, ActionSetBundle,
};

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

#[derive(Resource)]
struct SupportedProfiles(SupportedInteractionProfiles);

// TODO: filter interaction_profiles based on extensions/openxr version
fn create_bindings(
    interaction_profiles: Res<SupportedProfiles>,
    mut cmds: Commands,
    mut paths: ResMut<SubactionPaths>,
) {
    let set = cmds
        .spawn(ActionSetBundle::new(
            "suis_input_set",
            "Spatial Universal Interaction System Input Sources",
        ))
        .id();
    let mut req_paths = RequestedSubactionPaths::default();
    req_paths.push(paths.get_or_create_path("/oxr/user/hand/left", &mut cmds));
    req_paths.push(paths.get_or_create_path("/oxr/user/hand/right", &mut cmds));
    let bindings = binding_gen::spawn_bindings(set, &mut cmds, &interaction_profiles.0, &req_paths);
    fn get_bindings(
        binding: &'static str,
        profiles: &SupportedInteractionProfiles,
    ) -> OxrActionBlueprint {
        let mut blueprint = OxrActionBlueprint::default();
        for v in profiles.iter().map(|v| v.get_path()) {
            blueprint = blueprint.interaction_profile(v).binding(binding).end()
        }
        blueprint
    }
    let space_left = cmds
        .spawn((
            ActionBundle::new("method_pose_left", "Left Input Pose", set),
            SpaceActionValue::default(),
            get_bindings("/user/hand/left/input/aim/pose", &interaction_profiles.0),
        ))
        .id();
    let space_right = cmds
        .spawn((
            ActionBundle::new("method_pose_right", "Right Input Pose", set),
            SpaceActionValue::default(),
            get_bindings("/user/hand/right/input/aim/pose", &interaction_profiles.0),
        ))
        .id();
    cmds.insert_resource(bindings);
    cmds.insert_resource(SuisXrActions {
        set,
        space_left,
        space_right,
    });
}

#[derive(Clone, Copy, Debug, Resource)]
pub struct SuisXrActions {
    pub set: Entity,
    pub space_left: Entity,
    pub space_right: Entity,
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::world::CommandQueue, utils::hashbrown::HashSet};

    use super::*;

    #[test]
    fn check_for_non_init_actions() {
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut cmds = Commands::new(&mut queue, &world);

        let set = cmds
            .spawn(ActionSetBundle::new(
                "suis_input_set",
                "Spatial Universal Interaction System Input Sources",
            ))
            .id();
        let bindings = binding_gen::spawn_bindings(
            set,
            &mut cmds,
            &SupportedInteractionProfiles(HashSet::new()),
            &RequestedSubactionPaths::default(),
        );
        let w = bindings
            .iter_fields()
            .enumerate()
            .flat_map(|(i, v)| {
                let field_name = bindings.name_at(i).unwrap_or("NONE").to_string();
                if v.is::<Entity>() {
                    return vec![(field_name, v)];
                }
                println!("{}", v.reflect_type_path());
                if let bevy::reflect::ReflectRef::Struct(s) = v.reflect_ref() {
                    s.iter_fields()
                        .enumerate()
                        .map(|(i, v)| {
                            let str = format!("{field_name}.{}", s.name_at(i).unwrap_or("NONE"));
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
                v.downcast_ref::<Entity>()
                    .is_none_or(|v| *v == Entity::PLACEHOLDER)
            });
        assert!(!w);
    }
}

mod binding_gen {

    use super::*;
    use crate::{
        gen_bindings, xr_controllers::interaction_profiles::SupportedInteractionProfile as Profile,
    };
    use schminput::{
        openxr::OxrActionBlueprint, ActionBundle, BoolActionValue as Bool, F32ActionValue as F32,
        Vec2ActionValue as Vec2,
    };

    use super::XrControllerInputActions;
    gen_bindings!(
        spawn_bindings,
        (
            trigger.touched,
            "Trigger Touch",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/trigger/touch", "/user/hand/right/input/trigger/touch"),
                Profile::ViveFocus3  => ("/user/hand/left/input/trigger/touch", "/user/hand/right/input/trigger/touch"),
                Profile::OculusTouch => ("/user/hand/left/input/trigger/touch", "/user/hand/right/input/trigger/touch"),
                Profile::ValveIndex => ("/user/hand/left/input/trigger/touch", "/user/hand/right/input/trigger/touch")
            ]
        ),
        (
            trigger.pull,
            "Trigger Pull",
            F32,
            [
                Profile::Pico4 => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::ViveWand => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::ViveCosmos => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::ViveFocus3 => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::OculusTouch => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::ValveIndex => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::HpReverbG2 => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value")
            ]
        ),
        (
            trigger.pulled,
            "Trigger Pulled",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/trigger/click", "/user/hand/right/input/trigger/click"),
                Profile::ViveWand => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/click"),
                Profile::ViveCosmos => ("/user/hand/left/input/trigger/click", "/user/hand/right/input/trigger/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/trigger/click", "/user/hand/right/input/trigger/click"),
                Profile::OculusTouch => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value"),
                Profile::ValveIndex => ("/user/hand/left/input/trigger/click", "/user/hand/right/input/trigger/click"),
                Profile::HpReverbG2 => ("/user/hand/left/input/trigger/value", "/user/hand/right/input/trigger/value")
            ]
        ),
        (
            squeeze.value,
            "Squeeze",
            F32,
            [
                Profile::Pico4 => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value"),
                Profile::ViveFocus3 => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value"),
                Profile::OculusTouch => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value"),
                Profile::ValveIndex => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value"),
                Profile::HpReverbG2 => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value")
            ]
        ),
        (
            squeeze.squeezed,
            "Squeezed",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/squeeze/click", "/user/hand/right/input/squeeze/click"),
                Profile::ViveWand => ("/user/hand/left/input/squeeze/click", "/user/hand/right/input/squeeze/click"),
                Profile::ViveCosmos => ("/user/hand/left/input/squeeze/click", "/user/hand/right/input/squeeze/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/squeeze/click", "/user/hand/right/input/squeeze/click"),
                Profile::OculusTouch => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value"),
                Profile::ValveIndex => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value"),
                Profile::HpReverbG2 => ("/user/hand/left/input/squeeze/value", "/user/hand/right/input/squeeze/value")
            ]
        ),
        (
            squeeze.force,
            "Squeeze Force",
            F32,
            [
                Profile::ValveIndex => ("/user/hand/left/input/squeeze/force", "/user/hand/right/input/squeeze/force")
            ]
        ),
        (
            stick.pos,
            "Thumbstick Position",
            Vec2,
            [
                Profile::Pico4 => ("/user/hand/left/input/thumbstick", "/user/hand/right/input/thumbstick"),
                Profile::ViveCosmos => ("/user/hand/left/input/thumbstick", "/user/hand/right/input/thumbstick"),
                Profile::ViveFocus3 => ("/user/hand/left/input/thumbstick", "/user/hand/right/input/thumbstick"),
                Profile::OculusTouch => ("/user/hand/left/input/thumbstick", "/user/hand/right/input/thumbstick"),
                Profile::ValveIndex => ("/user/hand/left/input/thumbstick", "/user/hand/right/input/thumbstick"),
                Profile::HpReverbG2 => ("/user/hand/left/input/thumbstick", "/user/hand/right/input/thumbstick")
            ]
        ),
        (
            stick.touched,
            "Thumbstick Touched",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/thumbstick/touch", "/user/hand/right/input/thumbstick/touch"),
                Profile::ViveCosmos => ("/user/hand/left/input/thumbstick/touch", "/user/hand/right/input/thumbstick/touch"),
                Profile::ViveFocus3 => ("/user/hand/left/input/thumbstick/touch", "/user/hand/right/input/thumbstick/touch"),
                Profile::OculusTouch => ("/user/hand/left/input/thumbstick/touch", "/user/hand/right/input/thumbstick/touch"),
                Profile::ValveIndex => ("/user/hand/left/input/thumbstick/touch", "/user/hand/right/input/thumbstick/touch")
            ]
        ),
        (
            trackpad.pos,
            "Trackpad Position",
            Vec2,
            [
                Profile::ViveWand => ("/user/hand/left/input/trackpad", "/user/hand/right/input/trackpad"),
                Profile::ValveIndex => ("/user/hand/left/input/trackpad", "/user/hand/right/input/trackpad")
            ]
        ),
        (
            trackpad.pressed,
            "Trackpad Pressed",
            Bool,
            [
                Profile::ViveWand => ("/user/hand/left/input/trackpad/click", "/user/hand/right/input/trackpad/click"),
                Profile::ValveIndex => ("/user/hand/left/input/trackpad/force", "/user/hand/right/input/trackpad/force")
            ]
        ),
        (
            trackpad.touched,
            "Trackpad Touched",
            Bool,
            [
                Profile::ViveWand => ("/user/hand/left/input/trackpad/touch", "/user/hand/right/input/trackpad/touch"),
                Profile::ValveIndex => ("/user/hand/left/input/trackpad/touch", "/user/hand/right/input/trackpad/touch")
            ]
        ),
        (
            trackpad.force,
            "Trackpad Force",
            F32,
            [
                Profile::ValveIndex => ("/user/hand/left/input/trackpad/force", "/user/hand/right/input/trackpad/force")
            ]
        ),
        (
            button_north.pressed,
            "North Button Pressed",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/y/click", "/user/hand/right/input/b/click"),
                Profile::ViveCosmos => ("/user/hand/left/input/y/click", "/user/hand/right/input/b/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/y/click", "/user/hand/right/input/b/click"),
                Profile::OculusTouch => ("/user/hand/left/input/y/click", "/user/hand/right/input/b/click"),
                Profile::ValveIndex => ("/user/hand/left/input/b/click", "/user/hand/right/input/b/click"),
                Profile::HpReverbG2 => ("/user/hand/left/input/y/click", "/user/hand/right/input/b/click")
            ]
        ),
        (
            button_north.touched,
            "North Button Touched",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/y/touch", "/user/hand/right/input/b/touch"),
                Profile::OculusTouch => ("/user/hand/left/input/y/touch", "/user/hand/right/input/b/touch"),
                Profile::ValveIndex => ("/user/hand/left/input/b/touch", "/user/hand/right/input/b/touch")
            ]
        ),
        (
            button_south.pressed,
            "South Button Pressed",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/x/click", "/user/hand/right/input/a/click"),
                Profile::ViveCosmos => ("/user/hand/left/input/x/click", "/user/hand/right/input/a/click"),
                Profile::ViveFocus3 => ("/user/hand/left/input/x/click", "/user/hand/right/input/a/click"),
                Profile::OculusTouch => ("/user/hand/left/input/x/click", "/user/hand/right/input/a/click"),
                Profile::ValveIndex => ("/user/hand/left/input/a/click", "/user/hand/right/input/a/click"),
                Profile::HpReverbG2 => ("/user/hand/left/input/x/click", "/user/hand/right/input/a/click")
            ]
        ),
        (
            button_south.touched,
            "South Button Touched",
            Bool,
            [
                Profile::Pico4 => ("/user/hand/left/input/x/touch", "/user/hand/right/input/a/touch"),
                Profile::OculusTouch => ("/user/hand/left/input/x/touch", "/user/hand/right/input/a/touch"),
                Profile::ValveIndex => ("/user/hand/left/input/a/touch", "/user/hand/right/input/a/touch")
            ]
        ),
        (
            thumbrest_touched,
            "Thumbrest Touched",
            Bool,
            [
                Profile::ViveFocus3 => ("/user/hand/left/input/thumbrest/touch", "/user/hand/right/input/thumbrest/touch"),
                Profile::OculusTouch => ("/user/hand/left/input/thumbrest/touch", "/user/hand/right/input/thumbrest/touch")
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
                let mut blueprint = OxrActionBlueprint::default();
                $(
                if profiles.contains(&$profile) {
                    blueprint = blueprint.interaction_profile($profile.get_path())
                    $( .binding($binding) )*
                    .end();
                }
                )*
                let action_name = stringify!($($action).+).replace(".", "_");
                entities.$($action).+ = cmds.spawn((
                    ActionBundle {
                        paths: paths.clone(),
                        ..ActionBundle::new(action_name, $action_localized, set)
                    },
                    blueprint,
                    <$type>::default(),
                )).id();
            )*
            entities
        }
    };
}

pub fn make_placeholder_action_struct() -> XrControllerInputActions {
    XrControllerInputActions {
        trigger: TriggerActions {
            pull: Entity::PLACEHOLDER,
            pulled: Entity::PLACEHOLDER,
            touched: Entity::PLACEHOLDER,
        },
        squeeze: SqueezeActions {
            value: Entity::PLACEHOLDER,
            squeezed: Entity::PLACEHOLDER,
            force: Entity::PLACEHOLDER,
        },
        stick: StickActions {
            pos: Entity::PLACEHOLDER,
            touched: Entity::PLACEHOLDER,
        },
        trackpad: TrackpadActions {
            pos: Entity::PLACEHOLDER,
            pressed: Entity::PLACEHOLDER,
            touched: Entity::PLACEHOLDER,
            force: Entity::PLACEHOLDER,
        },
        button_north: TouchButtonActions {
            pressed: Entity::PLACEHOLDER,
            touched: Entity::PLACEHOLDER,
        },
        button_south: TouchButtonActions {
            pressed: Entity::PLACEHOLDER,
            touched: Entity::PLACEHOLDER,
        },
        thumbrest_touched: Entity::PLACEHOLDER,
    }
}

#[derive(Clone, Copy, Resource, Debug, Reflect)]
pub struct XrControllerInputActions {
    pub trigger: TriggerActions,
    pub squeeze: SqueezeActions,
    pub stick: StickActions,
    pub trackpad: TrackpadActions,
    // using north and south to add support for the leaked deckard controllers in the future
    pub button_north: TouchButtonActions,
    pub button_south: TouchButtonActions,

    pub thumbrest_touched: Entity,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct TriggerActions {
    /// f32
    pub pull: Entity,
    /// bool
    pub pulled: Entity,
    /// bool
    pub touched: Entity,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct SqueezeActions {
    /// f32
    pub value: Entity,
    /// bool
    pub squeezed: Entity,
    /// f32
    pub force: Entity,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct StickActions {
    /// Vec2
    pub pos: Entity,
    /// bool
    pub touched: Entity,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct TrackpadActions {
    /// Vec2
    pub pos: Entity,
    /// bool
    pub pressed: Entity,
    /// bool
    pub touched: Entity,
    /// f32
    pub force: Entity,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct TouchButtonActions {
    /// bool
    pub pressed: Entity,
    /// bool
    pub touched: Entity,
}
