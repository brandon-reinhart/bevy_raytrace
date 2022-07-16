use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(exit_on_esc);
    }
}

pub fn exit_on_esc(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ButtonState::Pressed && key_code == KeyCode::Escape {
                app_exit_events.send_default();
            }
        }
    }
}
