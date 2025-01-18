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
            .add_systems(Update, (read_button_input, read_motion));
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
    button_entries: HashMap<Action, ButtonInputCollection>, // TODO input context stack
    motion_entries: HashMap<Action, MotionEntry>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            button_entries: HashMap::<Action, ButtonInputCollection>::new(),
            motion_entries: HashMap::<Action, MotionEntry>::new(),
        }
    }
}

impl InputManager {
    pub fn register_motion(&mut self, action: Action, entries: Vec<MotionRegistryEntry>) {
        self.motion_entries
            .insert(action, MotionEntry(entries, Vec2::new(0., 0.)));
    }

    pub fn get_motion(&self, action: Action) -> Vec2 {
        if let Some(entry) = self.motion_entries.get(&action) {
            return entry.1;
        }
        unreachable!("Missing action: {}", action.0)
    }

    pub fn get_motion3z(&self, action: Action) -> Vec3 {
        let v2 = self.get_motion(action);
        Vec3 {
            x: v2.x,
            y: 0.0,
            z: v2.y,
        }
    }

    pub fn get_motion3y(&self, action: Action) -> Vec3 {
        let v2 = self.get_motion(action);
        Vec3 {
            x: v2.x,
            y: v2.y,
            z: 0.0,
        }
    }

    fn set_motion_from_gamepad_event(&mut self, axis_event: &GamepadAxisChangedEvent) {
        for motion_entry in self.motion_entries.values_mut() {
            let motion = &mut motion_entry.1;

            for mapping in motion_entry
                .0
                .iter()
                .filter(|mapping| mapping.0 == InputType::Gamepad)
            {
                for relation in mapping.1.iter().filter(|relation| {
                    if let MotionRelation::GamepadAxis(axis) = relation.1 {
                        return axis == axis_event.axis;
                    }
                    false
                }) {
                    match relation.0 {
                        MotionDirection::Up | MotionDirection::Down => motion.y = axis_event.value,
                        MotionDirection::Right | MotionDirection::Left => {
                            motion.x = axis_event.value
                        }
                    }
                }
            }
        }
    }

    pub fn register_button_events(&mut self, action: Action, buttons: Vec<Button>) {
        self.button_entries.insert(
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
        if let Some(entry) = self.button_entries.get(&action) {
            return !entry.pressed.is_empty();
        }
        false
    }

    pub fn is_action_just_pressed(&self, action: Action) -> bool {
        if let Some(entry) = self.button_entries.get(&action) {
            return !entry.just_pressed.is_empty();
        }
        false
    }

    pub fn is_action_just_released(&self, action: Action) -> bool {
        if let Some(entry) = self.button_entries.get(&action) {
            return !entry.just_released.is_empty();
        }
        false
    }

    fn set_button_pressed(&mut self, button: Button) {
        for buttoninput in self.button_entries.values_mut() {
            for b in buttoninput.released.extract_if(|b| *b == button) {
                buttoninput.just_pressed.insert(b);
            }

            // for b in buttoninput.just_pressed.extract_if(|b| *b == button) {
            //     buttoninput.pressed.insert(b);
            // }
        }
    }
    fn move_prev_frame_just_pressed(&mut self) {
        for buttoninput in self.button_entries.values_mut() {
            for b in buttoninput.just_pressed.drain() {
                buttoninput.pressed.insert(b);
            }
        }
    }

    fn set_button_released(&mut self, button: Button) {
        for buttoninput in self.button_entries.values_mut() {
            for b in buttoninput.pressed.extract_if(|b| *b == button) {
                buttoninput.just_released.insert(b);
            }
        }
    }

    fn move_prev_frame_just_released(&mut self) {
        for buttoninput in self.button_entries.values_mut() {
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
    for event in gamepad.read() {
        if let GamepadEvent::Axis(event) = event {
            input_manager.set_motion_from_gamepad_event(event);
        }
    }
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
    GamepadAxis(GamepadAxis),
    Mouse(MouseMotionDirection, i8),
}

pub struct MotionDirectionRelation(pub MotionDirection, pub MotionRelation);
pub struct MotionRegistryEntry(pub InputType, pub [MotionDirectionRelation; 4]);
struct MotionEntry(Vec<MotionRegistryEntry>, Vec2);
