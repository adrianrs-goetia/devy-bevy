use bevy::{
    app::AppExit,
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput},
    prelude::*,
};

pub struct ExitGamePlugin;

impl Plugin for ExitGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExitGameEvent>()
            .add_systems(Update, (keyboard_input, gamepad_input))
            .add_systems(Last, exit_game);
    }
}

#[derive(Event)]
struct ExitGameEvent;

fn exit_game(ev: EventReader<ExitGameEvent>, mut ev_exit: EventWriter<AppExit>) {
    if !ev.is_empty() {
        ev_exit.send(AppExit::Success);
    }
}

fn keyboard_input(
    mut input_events: EventReader<KeyboardInput>,
    mut ev: EventWriter<ExitGameEvent>,
) {
    for input in input_events.read() {
        if input.state.is_pressed() {
            match &input.key_code {
                KeyCode::Escape | KeyCode::CapsLock => {
                    ev.send(ExitGameEvent);
                }
                _ => continue,
            }
        }
    }
}

fn gamepad_input(
    mut gamepad_events: EventReader<GamepadEvent>,
    mut ev: EventWriter<ExitGameEvent>,
) {
    for event in gamepad_events.read() {
        match event {
            GamepadEvent::Button(button) => {
                if button.button == GamepadButton::Select && button.state.is_pressed() {
                    ev.send(ExitGameEvent);
                }
            }
            _ => continue,
        }
    }
}
