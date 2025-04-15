use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};

use crate::{
    input_method_data::InputMethodData, InputMethod, InputMethodActive, PointerInputMethod,
    SuisPreUpdateSets,
};

pub struct SuisWindowPointerPlugin;

impl Plugin for SuisWindowPointerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuisMouseConfig>();
        app.add_systems(
            PreUpdate,
            (update_input_method_ray, update_mouse_data)
                .in_set(SuisPreUpdateSets::UpdateInputMethods),
        );
        app.add_systems(PreStartup, manually_spawn_methods);
        app.add_observer(spawn_input_methods);
        app.add_observer(despawn_input_methods);
        app.add_observer(despawn_input_method_on_ref_remove);
    }
}

fn manually_spawn_methods(
    query: Query<Entity, (With<Window>, Without<SuisWindowCursor>)>,
    mut cmds: Commands,
) {
    for e in &query {
        spawn_method_on_entity(&mut cmds, e);
    }
}

fn spawn_method_on_entity(cmds: &mut Commands, e: Entity) {
    let method = cmds
        .spawn((
            InputMethod::new(),
            PointerInputMethod(Ray3d::new(Vec3::ZERO, Dir3::NEG_Z)),
            MouseInputMethod,
            InputMethodData::default(),
        ))
        .id();
    cmds.entity(e).insert(SuisWindowCursor(method));
}

fn despawn_input_method_on_ref_remove(
    t: Trigger<OnRemove, SuisWindowCursor>,
    mut cmds: Commands,
    has: Query<&SuisWindowCursor>,
) {
    if t.entity() == Entity::PLACEHOLDER {
        warn_once!("OnRemove Called with Placeholder entity?!");
        return;
    }
    let Ok(cursor) = has.get(t.entity()) else {
        warn!("very confused rn?!?!?!?!");
        return;
    };
    cmds.entity(cursor.0).despawn_recursive();
}

fn despawn_input_methods(
    t: Trigger<OnRemove, Window>,
    mut cmds: Commands,
    has: Query<&SuisWindowCursor>,
) {
    if t.entity() == Entity::PLACEHOLDER {
        warn_once!("OnRemove Called with Placeholder entity?!");
        return;
    }
    let Ok(cursor) = has.get(t.entity()) else {
        warn!("Removing Window without Input method?");
        return;
    };
    cmds.entity(cursor.0).despawn_recursive();
    cmds.entity(t.entity()).remove::<SuisWindowCursor>();
}
fn spawn_input_methods(
    t: Trigger<OnAdd, Window>,
    mut cmds: Commands,
    has: Query<Has<SuisWindowCursor>>,
) {
    info!("spawn");
    if t.entity() == Entity::PLACEHOLDER {
        warn_once!("OnAdd Called with Placeholder entity?!");
        return;
    }
    if has.get(t.entity()).unwrap_or(false) {
        warn!("New Window already has a Suis Cursor?!?! how?!");
    }
    spawn_method_on_entity(&mut cmds, t.entity());
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct MouseInputMethod;

#[derive(Resource)]
pub struct SuisMouseConfig {
    pub discrete_multiplier: f32,
    pub continuous_multiplier: f32,
}
impl Default for SuisMouseConfig {
    fn default() -> Self {
        SuisMouseConfig {
            discrete_multiplier: 0.02,
            continuous_multiplier: 0.002,
        }
    }
}

// doesn't handle multiple windows correctly
fn update_mouse_data(
    mut query: Query<&mut InputMethodData, (With<InputMethod>, With<MouseInputMethod>)>,
    mut scroll: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    config: Res<SuisMouseConfig>,
) {
    let mut discrete = Vec2::ZERO;
    let mut continuous = Vec2::ZERO;
    for e in scroll.read() {
        match e.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                discrete.x += e.x;
                discrete.y += e.y;
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                continuous.x += e.x;
                continuous.y += e.y;
            }
        }
    }
    for mut data in query.iter_mut() {
        data.select = buttons.pressed(MouseButton::Left) as u8 as f32;
        data.context = buttons.pressed(MouseButton::Middle) as u8 as f32;
        data.secondary = buttons.pressed(MouseButton::Right) as u8 as f32;
        data.grab = buttons.pressed(MouseButton::Forward) as u8 as f32;
        data.scroll = Some(
            (discrete * config.discrete_multiplier) + (continuous * config.continuous_multiplier),
        );
    }
}

fn update_input_method_ray(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform)>,
    windows: Query<(&Window, &SuisWindowCursor)>,
    mut input_method: Query<(
        &mut PointerInputMethod,
        &mut Transform,
        &mut InputMethodActive,
    )>,
) {
    let Ok(primary_window) = primary_window.get_single() else {
        warn_once!("no primary window?");
        return;
    };

    // this doesn't yet support multiple pointers per window, iirc that might be added in bevy 0.15
    for ((camera, cam_transform), window) in cams.iter().filter_map(|v| match v.0.target {
        RenderTarget::Window(w) => Some((v, w)),
        _ => None,
    }) {
        let window = match window {
            WindowRef::Primary => primary_window,
            WindowRef::Entity(e) => e,
        };
        let Ok((window, suis_cursor)) = windows.get(window) else {
            error_once!("Invalid window entity!");
            continue;
        };
        let Ok((mut method, mut transform, mut active)) = input_method.get_mut(suis_cursor.0)
        else {
            error!("unable to get input method for window");
            continue;
        };
        if let Some(pos) = window.cursor_position() {
            active.0 = true;
            if let Some(pos) = get_viewport_pos(pos, camera) {
                if let Ok(ray) = camera.viewport_to_world(cam_transform, pos) {
                    method.0 = ray;
                    transform.translation = ray.origin;
                    transform.look_at(ray.origin + *ray.direction, cam_transform.up());
                }
            }
        } else {
            active.0 = false;
        }
    }
}

#[derive(Clone, Copy, Component, Debug)]
struct SuisWindowCursor(Entity);

fn get_viewport_pos(logical_pos: Vec2, cam: &Camera) -> Option<Vec2> {
    if let Some(viewport_rect) = cam.logical_viewport_rect() {
        if !viewport_rect.contains(logical_pos) {
            return None;
        }
        Some(logical_pos - viewport_rect.min)
    } else {
        Some(logical_pos)
    }
}

#[derive(Clone, Copy, Component, Debug)]
pub struct WindowPointerInputMethodData {}
