use std::cmp::Ordering;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    input::mouse::MouseWheel,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};

use crate::{
    InputMethodDisabled, SuisPreUpdateSets,
    input_method::InputMethod,
    input_method_data::{NonSpatialInputData, SpatialInputData},
    order_helper::InputHandlerQueryHelper,
};

pub struct SuisWindowPointerRayPlugin;

impl Plugin for SuisWindowPointerRayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuisMouseConfig>();
        app.add_systems(
            PreUpdate,
            (update_input_method_ray, update_mouse_data)
                .chain()
                .in_set(SuisPreUpdateSets::UpdateInputMethods),
        );
        app.add_systems(
            PreUpdate,
            spawn_cursors.before(SuisPreUpdateSets::PrepareMethodEvents),
        );
    }
}

fn spawn_cursors(
    query: Query<Entity, (With<Window>, Without<SuisWindowCursor>)>,
    mut cmds: Commands,
) {
    for e in query {
        cmds.entity(e).insert(SuisWindowCursor(Entity::PLACEHOLDER));
    }
}

#[derive(Clone, Copy, Component, Debug)]
#[component(on_add = spawn_cursor_method)]
#[component(on_remove = despawn_cursor_method)]
struct SuisWindowCursor(Entity);

fn spawn_cursor_method(mut world: DeferredWorld, ctx: HookContext) {
    let method = world
        .commands()
        .spawn((
            InputMethod::new(),
            SpatialInputData::Ray(Ray3d::new(Vec3::ZERO, Dir3::NEG_Z)),
            MouseInputMethod,
            NonSpatialInputData::default(),
        ))
        .id();
    world
        .commands()
        .entity(ctx.entity)
        .insert(SuisWindowCursor(method));
}
fn despawn_cursor_method(mut world: DeferredWorld, ctx: HookContext) {
    if let Some(SuisWindowCursor(method)) =
        world.entity(ctx.entity).get::<SuisWindowCursor>().copied()
    {
        world.commands().entity(method).despawn();
    }
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
    mut query: Query<
        (
            &mut NonSpatialInputData,
            &mut InputMethod,
            &SpatialInputData,
        ),
        With<MouseInputMethod>,
    >,
    mut scroll: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    config: Res<SuisMouseConfig>,
    handler_query: InputHandlerQueryHelper,
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
    for (mut data, mut input_method, spatial_data) in query.iter_mut() {
        data.select = buttons.pressed(MouseButton::Left) as u8 as f32;
        data.context = buttons.pressed(MouseButton::Middle) as u8 as f32;
        data.secondary = buttons.pressed(MouseButton::Right) as u8 as f32;
        data.grab = buttons.pressed(MouseButton::Forward) as u8 as f32;
        data.scroll = Some(
            (discrete * config.discrete_multiplier) + (continuous * config.continuous_multiplier),
        );
        let mut handlers =
            handler_query.query_all_handler_fields(|(handler, field, field_transform)| {
                (handler, spatial_data.distance(field, field_transform))
            });

        handlers.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        let handlers = handlers.into_iter().map(|(e, _)| e).collect();
        input_method.set_handler_order(handlers);
    }
}

fn update_input_method_ray(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform)>,
    windows: Query<(&Window, &SuisWindowCursor)>,
    mut input_method: Query<
        (&mut SpatialInputData, Has<InputMethodDisabled>),
        With<MouseInputMethod>,
    >,
    mut cmds: Commands,
) {
    let Ok(primary_window) = primary_window.single() else {
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
        let Ok((mut method, disabled)) = input_method.get_mut(suis_cursor.0) else {
            error!("unable to get input method for window");
            continue;
        };
        if let Some(pos) = window.cursor_position() {
            if disabled {
                cmds.entity(suis_cursor.0).remove::<InputMethodDisabled>();
            }
            if let Some(pos) = get_viewport_pos(pos, camera) {
                if let Ok(ray) = camera.viewport_to_world(cam_transform, pos) {
                    *method = SpatialInputData::Ray(ray);
                }
            }
        } else if !disabled {
            cmds.entity(suis_cursor.0).insert(InputMethodDisabled);
        }
    }
}

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
