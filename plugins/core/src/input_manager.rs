use bevy::core_pipeline::core_2d::graph::input;
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadInput};
use bevy::input::mouse::MouseButtonInput;
use bevy::math::vec2;
use bevy::utils::default;
use bevy::utils::hashbrown::HashSet;
use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::MouseMotion, ButtonState},
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

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Action(pub &'static str);

#[derive(PartialEq, Eq, Hash)]
pub enum InputType {
    Mouse,
    Keyboard,
    Gamepad,
}

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
    pub fn register_motion(
        &mut self,
        action: Action,
        // entries: MotionRegisterEntry,
        entries: (InputType, [MotionEntry; 4]),
        // entries: Vec<InputType, [MotionEntry; 4]>,
        // entrytype: InputType,
        // motions: [MotionEntry; 4],
    ) {
    }

    pub fn get_motion(&self, action: &Action) -> Vec2 {
        Vec2::new(0., 0.)
    }

    pub fn get_motion3z(&self, action: &Action) -> Vec3 {
        let v2 = self.get_motion(action);
        Vec3 {
            x: v2.x,
            y: 0.0,
            z: v2.y,
        }
    }

    pub fn get_motion3y(&self, action: &Action) -> Vec3 {
        let v2 = self.get_motion(action);
        Vec3 {
            x: v2.x,
            y: v2.y,
            z: 0.0,
        }
    }

    fn set_motion_from_gamepad(&mut self, motion: GamepadAxisChangedEvent) {}

    pub fn register_button_events(&mut self, action: Action, buttons: Vec<Button>) {
        self.entries.insert(
            action,
            ButtonInputCollection {
                just_pressed: HashSet::<Button>::new(),
                pressed: HashSet::<Button>::new(),
                just_released: HashSet::<Button>::new(),
                released: buttons.into_iter().collect::<HashSet<_>>(),
            },
        );
    }

    pub fn is_action_pressed(&self, action: Action) -> bool {
        if let Some(entry) = self.entries.get(&action) {
            return !entry.pressed.is_empty();
        }
        false
    }

    pub fn is_action_just_pressed(&self, action: Action) -> bool {
        if let Some(entry) = self.entries.get(&action) {
            return !entry.just_pressed.is_empty();
        }
        false
    }

    pub fn is_action_just_released(&self, action: Action) -> bool {
        if let Some(entry) = self.entries.get(&action) {
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
    mut mouse: EventReader<MouseButtonInput>,
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

    for event in mouse.read() {
        match event.state {
            ButtonState::Pressed => input_manager.set_button_pressed(Button::Mouse(event.button)),
            ButtonState::Released => input_manager.set_button_released(Button::Mouse(event.button)),
        }
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

fn read_motion(
    // mouse:
    mut gamepad: EventReader<GamepadEvent>,
    mut input_manager: ResMut<InputManager>,
) {
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Button {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    Gamepad(GamepadButton),
}

struct ButtonInputCollection {
    just_pressed: HashSet<Button>,
    pressed: HashSet<Button>,
    just_released: HashSet<Button>,
    released: HashSet<Button>,
}

// Right/Left   x-axis
// Up/Down      y-axis
pub enum MotionDirection {
    Up,
    Down,
    Right,
    Left,
}

pub enum MouseMotionDirection {
    Up,
    Down,
    Right,
    Left,
}

pub enum MotionRelation {
    Gamepad(GamepadAxis, i8),
    Mouse(MouseMotionDirection, i8),
}

pub struct MotionEntry(pub MotionDirection, pub MotionRelation);
