pub mod default_bindings;
pub mod input_method_data;
mod query_ext;
use bevy::prelude::*;
use bevy_suis::xr::HandSide;
use bevy_suis::InputMethod;
use default_bindings::XrControllerInputActions;
use input_method_data::Squeeze;
use input_method_data::Stick;
use input_method_data::TouchButton;
use input_method_data::Trackpad;
use query_ext::ActionValueQueryExt as _;
use schminput::prelude::*;
use input_method_data::XrControllerInputMethodData;
use input_method_data::Trigger;
use schminput::subaction_paths::SubactionPath;

fn update_method_data(
    vec2: Query<&Vec2ActionValue>,
    f32: Query<&F32ActionValue>,
    bool: Query<&BoolActionValue>,
    actions: Res<XrControllerInputActions>,
    mut method_query: Query<(&mut XrControllerInputMethodData, &HandSide), With<InputMethod>>,
    mut paths: ResMut<SubactionPaths>,
    mut cmds: Commands,
) {
    fn get_data(
        vec2: &Query<&Vec2ActionValue>,
        f32: &Query<&F32ActionValue>,
        bool: &Query<&BoolActionValue>,
        actions: &XrControllerInputActions,
        path: &SubactionPath,
    ) -> XrControllerInputMethodData {
        XrControllerInputMethodData {
            trigger: Trigger {
                pull: f32.get_with_path_or_default(actions.trigger.pull, path),
                pulled: bool.get_with_path_or_default(actions.trigger.pulled, path),
                touched: bool.get_with_path_or_default(actions.trigger.touched, path),
            },
            squeeze: Squeeze {
                value: f32.get_with_path_or_default(actions.squeeze.value, path),
                squeezed: bool.get_with_path_or_default(actions.squeeze.squeezed, path),
                force: f32.get_with_path_or_default(actions.squeeze.force, path),
            },
            stick: Stick {
                pos: vec2.get_with_path_or_default(actions.stick.pos, path),
                touched: bool.get_with_path_or_default(actions.stick.touched, path),
            },
            trackpad: Trackpad {
                pos: vec2.get_with_path_or_default(actions.trackpad.pos, path),
                pressed: bool.get_with_path_or_default(actions.trackpad.pressed, path),
                touched: bool.get_with_path_or_default(actions.trackpad.touched, path),
                force: f32.get_with_path_or_default(actions.trackpad.force, path),
            },
            button_north: TouchButton {
                pressed: bool.get_with_path_or_default(actions.button_north.pressed, path),
                touched: bool.get_with_path_or_default(actions.button_north.touched, path),
            },
            button_south: TouchButton {
                pressed: bool.get_with_path_or_default(actions.button_south.pressed, path),
                touched: bool.get_with_path_or_default(actions.button_south.touched, path),
            },
            thumbrest_touched: bool.get_with_path_or_default(actions.thumbrest_touched, path),
        }
    }
    let action_data_left = get_data(
        &vec2,
        &f32,
        &bool,
        &actions,
        &paths.get_or_create_path("/oxr/user/hand/left", &mut cmds),
    );
    let action_data_right = get_data(
        &vec2,
        &f32,
        &bool,
        &actions,
        &paths.get_or_create_path("/oxr/user/hand/right", &mut cmds),
    );
    for (mut data, side) in &mut method_query {
        *data = match side {
            HandSide::Left => action_data_left,
            HandSide::Right => action_data_right,
        };
    }
}
