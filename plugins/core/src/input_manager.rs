use bevy::core_pipeline::core_2d::graph::input;
use bevy::input::gamepad::GamepadInput;
use bevy::utils::default;
use bevy::utils::hashbrown::HashSet;
use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::MouseMotion},
    prelude::*,
};
use std::collections::HashMap;

pub struct InputManagerPlugin;

impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputManager::default())
            .add_systems(Update, read_button_input);
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Action(&'static str);

/// Manager
#[derive(Resource)]
pub struct InputManager {
    entries: HashMap<Action, ButtonInputCollection>, // TODO input context stack
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            entries: HashMap::<Action, ButtonInputCollection>::new(),
        }
    }
}

impl InputManager {
    pub fn register_button_events(&mut self, action: &'static str, buttons: Vec<Button>) {
        self.entries.insert(
            Action(action),
            ButtonInputCollection {
                just_pressed: HashSet::<Button>::new(),
                pressed: HashSet::<Button>::new(),
                just_released: HashSet::<Button>::new(),
                released: buttons.into_iter().collect::<HashSet<_>>(),
            },
        );
    }

    pub fn is_action_pressed(&self, action: &'static str) -> bool {
        if let Some(entry) = self.entries.get(&Action(action)) {
            return !entry.pressed.is_empty();
        }
        false
    }

    pub fn is_action_just_pressed(&self, action: &'static str) -> bool {
        if let Some(entry) = self.entries.get(&Action(action)) {
            return !entry.just_pressed.is_empty();
        }
        false
    }

    pub fn is_action_just_released(&self, action: &'static str) -> bool {
        if let Some(entry) = self.entries.get(&Action(action)) {
            return !entry.just_released.is_empty();
        }
        false
    }

    fn set_button_pressed(&mut self, button: Button) {
        for buttoninput in self.entries.values_mut() {

            for b in buttoninput.released.extract_if(|b| *b == button) {
                buttoninput.just_pressed.insert(b);
            }

            // for b in buttoninput.just_pressed.extract_if(|b| *b == button) {
            //     buttoninput.pressed.insert(b);
            // }
        }
    }
    fn move_prev_frame_just_pressed(&mut self) {
        for buttoninput in self.entries.values_mut() {
            for b in buttoninput.just_pressed.drain() {
                buttoninput.pressed.insert(b);
            }
        }
    }

    fn set_button_released(&mut self, button: Button) {
        for buttoninput in self.entries.values_mut() {
            for b in buttoninput.pressed.extract_if(|b| *b == button) {
                buttoninput.just_released.insert(b);
            }
        }
    }

    fn move_prev_frame_just_released(&mut self) {
        for buttoninput in self.entries.values_mut() {
            for b in buttoninput.just_released.drain() {
                buttoninput.released.insert(b);
            }
        }
    }
}

fn read_button_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gamepad: EventReader<GamepadEvent>,
    mut input_manager: ResMut<InputManager>,
) {
    input_manager.move_prev_frame_just_pressed();
    input_manager.move_prev_frame_just_released();

    for key in keyboard.get_just_pressed() {
        input_manager.set_button_pressed(Button::Keyboard(*key));
    }
    for key in keyboard.get_just_released() {
        input_manager.set_button_released(Button::Keyboard(*key));
    }

    for event in gamepad.read() {
        match event {
            GamepadEvent::Button(button) => {
                if button.state.is_pressed() {
                    input_manager.set_button_pressed(Button::Gamepad(button.button));
                } else {
                    input_manager.set_button_released(Button::Gamepad(button.button));
                }
            }
            _ => (),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Button {
    Keyboard(KeyCode),
    Gamepad(GamepadButton),
}

struct ButtonInputCollection {
    just_pressed: HashSet<Button>,
    pressed: HashSet<Button>,
    just_released: HashSet<Button>,
    released: HashSet<Button>,
}
