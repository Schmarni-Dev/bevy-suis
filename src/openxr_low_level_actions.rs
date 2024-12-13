use std::{borrow::Cow, mem};

use bevy::{prelude::*, utils::HashMap};
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    resources::OxrInstance,
    session::OxrSession,
};
use bevy_mod_xr::{
    session::{session_available, XrPreDestroySession, XrSessionCreated},
    spaces::XrSpace,
    types::XrPose,
};
use openxr::{ActionTy, Path, Vector2f};

use self::binding::{SuisOxrBindingAction, SuisOxrBindingSet, SuisOxrBindings};

pub struct SuisOxrActionPlugin;
impl Plugin for SuisOxrActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuisOxrBindings>();
        app.add_systems(
            PreUpdate,
            (
                clear_last_action_values,
                sync_sets.before(OxrActionSetSyncSet),
            )
                .chain(),
        );
        app.add_systems(PostStartup, create_actions.run_if(session_available));
        // The .after should ideally not be needed, but it is for bevy_mod_openxr 0.1.0-rc1
        app.add_systems(XrSessionCreated, attach_actions);
        // There might be a ordering issue here too
        app.add_systems(XrPreDestroySession, destroy_action_spaces);
        app.add_systems(OxrSendActionBindings, suggest_bindings);
    }
}
fn attach_actions(sets: Query<&SuisOxrActionSet>, mut writer: EventWriter<OxrAttachActionSet>) {
    writer.send_batch(sets.iter().map(|v| &v.set).cloned().map(OxrAttachActionSet));
}

fn suggest_bindings(
    sets: Query<(&SuisOxrActionSet, &SuisOxrBindingSet)>,
    mut writer: EventWriter<OxrSuggestActionBinding>,
) {
    for (set, bindings) in sets.iter() {
        for (action, binding) in set.actions.iter().zip(bindings.actions.iter()) {
            for (profile, bindings) in binding.bindings.iter() {
                writer.send(OxrSuggestActionBinding {
                    action: action.as_raw(),
                    interaction_profile: Cow::Borrowed(*profile),
                    bindings: bindings.iter().map(|v| Cow::Borrowed(*v)).collect(),
                });
            }
        }
    }
}

fn create_actions(
    mut cmds: Commands,
    instance: Res<OxrInstance>,
    mut bindings: ResMut<SuisOxrBindings>,
) {
    let mut sets = Vec::<Entity>::new();
    let bindings = mem::take(&mut *bindings);
    for set_bindings in bindings.sets {
        let set =
            match instance.create_action_set(set_bindings.name, set_bindings.localized_name, 0) {
                Ok(v) => v,
                Err(err) => {
                    warn!("error while creating action_set: {}", err);
                    continue;
                }
            };
        let mut actions = Vec::<SuisOxrAction>::new();
        for action in set_bindings.actions.iter() {
            // TODO: implement subaction paths
            let a = match create_suis_oxr_action(action, &set) {
                Ok(a) => a,
                Err(err) => {
                    warn!("error while creating action: {}", err);
                    continue;
                }
            };
            actions.push(a);
        }
        sets.push(
            cmds.spawn(SuisOxrActionSet {
                name: set_bindings.name,
                localized_name: set_bindings.localized_name,
                active: true,
                set,
                actions,
            })
            .insert(set_bindings)
            .id(),
        );
    }

    cmds.remove_resource::<SuisOxrBindings>();
}
fn create_suis_oxr_action(
    action: &SuisOxrBindingAction,
    set: &openxr::ActionSet,
) -> openxr::Result<SuisOxrAction> {
    let a = match action.action_type {
        binding::SuisOxrActionType::Bool => SuisOxrAction::Bool(SuisOxrTypedAction {
            action: set.create_action(action.name, action.localized_name, &[])?,
            last_values: HashMap::new(),
            current_values: HashMap::new(),
        }),
        binding::SuisOxrActionType::F32 => SuisOxrAction::F32(SuisOxrTypedAction {
            action: set.create_action(action.name, action.localized_name, &[])?,
            last_values: HashMap::new(),
            current_values: HashMap::new(),
        }),
        binding::SuisOxrActionType::Vec2 => SuisOxrAction::Vec2(SuisOxrTypedAction {
            action: set.create_action(action.name, action.localized_name, &[])?,
            last_values: HashMap::new(),
            current_values: HashMap::new(),
        }),
        binding::SuisOxrActionType::Space => SuisOxrAction::Space(SuisOxrSpaceAction {
            action: set.create_action(action.name, action.localized_name, &[])?,
            last_values: HashMap::new(),
        }),
    };
    Ok(a)
}

fn destroy_action_spaces(mut sets: Query<&mut SuisOxrActionSet>, session: Res<OxrSession>) {
    for mut set in sets.iter_mut() {
        for action in set.actions.iter_mut() {
            if let SuisOxrAction::Space(a) = action {
                for space in mem::take(&mut a.last_values).into_values() {
                    let _ = session.destroy_space(space);
                }
            }
        }
    }
}

fn clear_last_action_values(mut sets: Query<&mut SuisOxrActionSet>) {
    for mut set in sets.iter_mut() {
        for action in set.actions.iter_mut() {
            match action {
                SuisOxrAction::Bool(a) => a.clear_last_values(),
                SuisOxrAction::F32(a) => a.clear_last_values(),
                SuisOxrAction::Vec2(a) => a.clear_last_values(),
                SuisOxrAction::Space(_) => (),
            }
        }
    }
}

fn sync_sets(sets: Query<&SuisOxrActionSet>, mut writer: EventWriter<OxrSyncActionSet>) {
    writer.send_batch(
        sets.iter()
            .filter(|v| v.active)
            .map(|v| &v.set)
            .cloned()
            .map(OxrSyncActionSet),
    );
}

pub mod binding {
    use bevy::{
        ecs::{component::Component, system::Resource},
        utils::HashMap,
    };

    #[derive(Clone)]
    pub enum SuisOxrActionType {
        Bool,
        F32,
        Vec2,
        Space,
    }
    // Might need a custom default impl for default bindings?
    #[derive(Resource, Default)]
    pub struct SuisOxrBindings {
        pub(super) sets: Vec<SuisOxrBindingSet>,
    }
    impl SuisOxrBindings {
        pub fn add_set(
            self,
            name: &'static str,
            pretty_name: &'static str,
        ) -> SuisOxrBindingSetBuilder {
            SuisOxrBindingSetBuilder {
                bindings: self,
                set: SuisOxrBindingSet {
                    name,
                    localized_name: pretty_name,
                    actions: Vec::new(),
                },
            }
        }
    }
    #[derive(Component)]
    pub struct SuisOxrBindingSet {
        pub(super) name: &'static str,
        pub(super) localized_name: &'static str,
        pub(super) actions: Vec<SuisOxrBindingAction>,
    }
    pub struct SuisOxrBindingAction {
        pub(super) action_type: SuisOxrActionType,
        pub(super) name: &'static str,
        pub(super) localized_name: &'static str,
        pub(super) bindings: HashMap<&'static str, Vec<&'static str>>,
    }

    pub struct SuisOxrBindingSetBuilder {
        bindings: SuisOxrBindings,
        set: SuisOxrBindingSet,
    }
    impl SuisOxrBindingSetBuilder {
        pub fn add_action_bool(
            self,
            name: &'static str,
            pretty_name: &'static str,
        ) -> SuisOxrBindingActionBuilder {
            SuisOxrBindingActionBuilder {
                set_builder: self,
                action: SuisOxrBindingAction {
                    action_type: SuisOxrActionType::Bool,
                    name,
                    localized_name: pretty_name,
                    bindings: HashMap::new(),
                },
            }
        }
        pub fn add_action_f32(
            self,
            name: &'static str,
            pretty_name: &'static str,
        ) -> SuisOxrBindingActionBuilder {
            SuisOxrBindingActionBuilder {
                set_builder: self,
                action: SuisOxrBindingAction {
                    action_type: SuisOxrActionType::F32,
                    name,
                    localized_name: pretty_name,
                    bindings: HashMap::new(),
                },
            }
        }
        pub fn add_action_vec2(
            self,
            name: &'static str,
            pretty_name: &'static str,
        ) -> SuisOxrBindingActionBuilder {
            SuisOxrBindingActionBuilder {
                set_builder: self,
                action: SuisOxrBindingAction {
                    action_type: SuisOxrActionType::Vec2,
                    name,
                    localized_name: pretty_name,
                    bindings: HashMap::new(),
                },
            }
        }
        pub fn add_action_xr_space(
            self,
            name: &'static str,
            pretty_name: &'static str,
        ) -> SuisOxrBindingActionBuilder {
            SuisOxrBindingActionBuilder {
                set_builder: self,
                action: SuisOxrBindingAction {
                    action_type: SuisOxrActionType::Space,
                    name,
                    localized_name: pretty_name,
                    bindings: HashMap::new(),
                },
            }
        }
        pub fn finish(mut self) -> SuisOxrBindings {
            self.bindings.sets.push(self.set);
            self.bindings
        }
    }
    pub struct SuisOxrBindingActionBuilder {
        set_builder: SuisOxrBindingSetBuilder,
        action: SuisOxrBindingAction,
    }
    impl SuisOxrBindingActionBuilder {
        pub fn add_profile_bindings(
            self,
            interaction_profile: &'static str,
        ) -> SuisOxrBindingActionProfileBuilder {
            SuisOxrBindingActionProfileBuilder {
                action_builder: self,
                profile: interaction_profile,
            }
        }
        pub fn finish(mut self) -> SuisOxrBindingSetBuilder {
            self.set_builder.set.actions.push(self.action);
            self.set_builder
        }
    }
    pub struct SuisOxrBindingActionProfileBuilder {
        action_builder: SuisOxrBindingActionBuilder,
        profile: &'static str,
    }
    impl SuisOxrBindingActionProfileBuilder {
        pub fn add_binding(mut self, path: &'static str) -> Self {
            self.action_builder
                .action
                .bindings
                .entry(self.profile)
                .or_default()
                .push(path);
            self
        }
        pub fn finish(self) -> SuisOxrBindingActionBuilder {
            self.action_builder
        }
    }
}

#[derive(Component)]
pub struct SuisOxrActionSet {
    pub name: &'static str,
    pub localized_name: &'static str,
    pub active: bool,
    pub set: openxr::ActionSet,
    pub actions: Vec<SuisOxrAction>,
}
#[derive(Clone)]
pub enum SuisOxrAction {
    Bool(SuisOxrTypedAction<bool>),
    F32(SuisOxrTypedAction<f32>),
    Vec2(SuisOxrTypedAction<Vector2f>),
    Space(SuisOxrSpaceAction),
}
impl SuisOxrAction {
    pub fn as_raw(&self) -> openxr::sys::Action {
        match self {
            SuisOxrAction::Bool(v) => v.action.as_raw(),
            SuisOxrAction::F32(v) => v.action.as_raw(),
            SuisOxrAction::Vec2(v) => v.action.as_raw(),
            SuisOxrAction::Space(v) => v.action.as_raw(),
        }
    }
}

#[derive(Clone)]
pub struct SuisOxrTypedAction<T: ActionTy> {
    pub action: openxr::Action<T>,
    last_values: HashMap<Path, T>,
    current_values: HashMap<Path, T>,
}

#[derive(Clone)]
pub struct SuisOxrSpaceAction {
    pub action: openxr::Action<openxr::Posef>,
    last_values: HashMap<Path, XrSpace>,
}

impl SuisOxrSpaceAction {
    pub fn get_space(
        &mut self,
        session: &OxrSession,
        path: Path,
        offset: XrPose,
    ) -> openxr::Result<XrSpace> {
        let space = session.create_action_space(&self.action, path, offset)?;
        let last_space = self.last_values.insert(path, space);
        if last_space == Some(space) {
            let _ = session.destroy_space(last_space.expect("None =! Some(XrSpace)"));
        }
        Ok(space)
    }
}
impl SuisOxrTypedAction<bool> {
    fn clear_last_values(&mut self) {
        mem::swap(&mut self.last_values, &mut self.current_values);
        self.current_values.clear()
    }
    pub fn get_current_value(&mut self, session: &OxrSession, path: Path) -> bool {
        let state = self.action.state(session, path).unwrap();
        if !self.current_values.contains_key(&path) {
            self.current_values.insert(path, state.current_state);
        }
        state.current_state
    }
    pub fn get_last_value(&self, path: Path) -> Option<bool> {
        self.last_values.get(&path).copied()
    }
}
impl SuisOxrTypedAction<f32> {
    fn clear_last_values(&mut self) {
        mem::swap(&mut self.last_values, &mut self.current_values);
        self.current_values.clear()
    }
    pub fn get_current_value(&mut self, session: &OxrSession, path: Path) -> f32 {
        let state = self.action.state(session, path).unwrap();
        if !self.current_values.contains_key(&path) {
            self.current_values.insert(path, state.current_state);
        }
        state.current_state
    }
    pub fn get_last_value(&self, path: Path) -> Option<f32> {
        self.last_values.get(&path).copied()
    }
}
impl SuisOxrTypedAction<Vector2f> {
    fn clear_last_values(&mut self) {
        mem::swap(&mut self.last_values, &mut self.current_values);
        self.current_values.clear()
    }
    pub fn get_current_value(&mut self, session: &OxrSession, path: Path) -> Vector2f {
        let state = self.action.state(session, path).unwrap();
        if !self.last_values.contains_key(&path) {
            self.last_values.insert(path, state.current_state);
        }
        state.current_state
    }
    pub fn get_last_value(&self, path: Path) -> Option<Vector2f> {
        self.last_values.get(&path).copied()
    }
}
