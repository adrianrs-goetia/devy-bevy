use bevy::{
    app::AppExit,
    prelude::*,
};

use crate::input_manager as input;

pub struct ExitGamePlugin;

impl Plugin for ExitGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExitGameEvent>()
            .add_systems(Startup, register_input)
            .add_systems(Update, read_input)
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

static EXIT_GAME: input::Action = input::Action("exit_game");

fn register_input(mut im: ResMut<input::InputManager>) {
    im.register_action_button(
        EXIT_GAME,
        vec![
            input::button::Variant::Keyboard(KeyCode::Escape),
            input::button::Variant::Keyboard(KeyCode::CapsLock),
            input::button::Variant::Gamepad(GamepadButton::Select),
        ],
    );
}

fn read_input(im: Res<input::InputManager>, mut ev: EventWriter<ExitGameEvent>) {
    if im.is_action_just_pressed(EXIT_GAME) {
        ev.send(ExitGameEvent);
    }
}
