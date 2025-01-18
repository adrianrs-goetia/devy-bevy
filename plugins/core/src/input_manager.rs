use bevy::core_pipeline::core_2d::graph::input;
use bevy::input::gamepad::{GamepadAxisChangedEvent, GamepadInput};
use bevy::input::mouse::{AccumulatedMouseMotion, MouseButtonInput};
use bevy::math::vec2;
use bevy::utils::default;
use bevy::utils::hashbrown::HashSet;
use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::MouseMotion, ButtonState},
    prelude::*,
};
use motion::ActionEntry;
use std::collections::HashMap;

pub struct InputManagerPlugin;

impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputManager::default())
            .add_systems(Update, (read_button_input, motion::read_motion));
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Action(pub &'static str);

#[derive(PartialEq, Eq, Hash)]
pub enum InputType {
    Keyboard,
    Mouse,
    Gamepad,
}

/// Manager
#[derive(Resource)]
pub struct InputManager {
    button_entries: HashMap<Action, ButtonInputCollection>, // TODO input context stack
    motion_entries: HashMap<Action, motion::ActionEntry>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            button_entries: HashMap::<Action, ButtonInputCollection>::new(),
            motion_entries: HashMap::<Action, ActionEntry>::new(),
        }
    }
}

impl InputManager {
    pub fn register_motion(&mut self, action: Action, entries: Vec<motion::Entry>) {
        self.motion_entries.insert(
            action,
            motion::ActionEntry {
                motion_entries: entries,
                motion: Vec2::new(0., 0.),
            },
        );
    }

    pub fn get_motion(&self, action: Action) -> Vec2 {
        if let Some(entry) = self.motion_entries.get(&action) {
            return entry.motion;
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

    fn set_motion(
        &mut self,
        axis_events: Vec<GamepadAxisChangedEvent>,
        mouse_motion: Option<Vec2>,
        keyboard: motion::Keycode,
    ) {
        for motion_entry in self.motion_entries.values_mut() {
            // motion_entry.motion = Vec2::new(0., 0.); // reset every frame

            for mapping in motion_entry.motion_entries.iter() {
                match mapping.input_type {
                    InputType::Gamepad => {
                        for axis_event in axis_events.iter() {
                            for relation in mapping.relations.iter().filter(|relation| {
                                if let motion::Relation::GamepadAxis(axis, _) = *relation {
                                    return *axis == axis_event.axis;
                                }
                                false
                            }) {
                                if let motion::Relation::GamepadAxis(_, axis_value) = relation {
                                    match axis_value {
                                        motion::Axis::Y => motion_entry.motion.y = axis_event.value,
                                        motion::Axis::X => motion_entry.motion.x = axis_event.value,
                                        _ => (),
                                    }
                                }
                            }
                        }
                    }
                    InputType::Mouse => {
                        // TODO, properly reset motion after stopping mouse motion
                        if let Some(mouse_motion) = mouse_motion {
                            // messy with multiple entries for a type
                            motion_entry.motion =
                                mouse_motion / motion::MOUSE_MOTION_SENSITIVITY as f32;
                        }
                    }
                    InputType::Keyboard => {
                        let mut new_motion = Vec2::ZERO;
                        for relation in mapping.relations.iter() {
                            if let motion::Relation::KeyCode(key, val) = relation {
                                let newval = if keyboard.is_key_pressed(*key) {
                                    val.get_value()
                                } else {
                                    0.0
                                };
                                match val {
                                    motion::Axis::PosY | motion::Axis::NegY => {
                                        new_motion.y += newval
                                    }
                                    motion::Axis::PosX | motion::Axis::NegX => {
                                        new_motion.x += newval
                                    }
                                    _ => (),
                                }
                            }
                        }
                        motion_entry.motion = new_motion
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

//  ha ting i moduler

pub mod motion {
    use bevy::prelude::*;
    use bevy::{
        input::{
            gamepad::{GamepadAxisChangedEvent, GamepadEvent},
            mouse::AccumulatedMouseMotion,
            ButtonInput,
        },
        math::Vec2,
        prelude::{EventReader, GamepadAxis, KeyCode, Res, ResMut},
        utils::HashSet,
    };

    pub enum Axis {
        X,
        Y,
        PosX,
        NegX,
        PosY,
        NegY,
    }

    impl Axis {
        pub fn get_value(&self) -> f32 {
            match self {
                Self::PosX | Self::PosY => 1.0,
                Self::NegX | Self::NegY => -1.0,
                _ => 0.0,
            }
        }
    }

    pub const MOUSE_MOTION_SENSITIVITY: i8 = 30; // Normalizing mouse motion

    pub enum Relation {
        GamepadAxis(GamepadAxis, Axis),
        Mouse,
        KeyCode(KeyCode, Axis),
    }

    pub struct Entry {
        pub input_type: super::InputType,
        pub relations: Vec<Relation>,
    }

    pub struct ActionEntry {
        pub motion_entries: Vec<Entry>,
        pub motion: Vec2,
    }

    pub(super) struct Keycode {
        pressed: HashSet<KeyCode>,
    }

    impl Keycode {
        pub(super) fn is_key_pressed(&self, key: KeyCode) -> bool {
            return self.pressed.contains(&key);
        }
    }

    pub(super) fn read_motion(
        mouse_motion: Res<AccumulatedMouseMotion>,
        keyboard: Res<ButtonInput<KeyCode>>,
        mut gamepad: EventReader<GamepadEvent>,
        mut input_manager: ResMut<super::InputManager>,
    ) {
        let gamepad_axis_events = {
            let mut events = Vec::<GamepadAxisChangedEvent>::new();
            for event in gamepad.read().into_iter() {
                if let GamepadEvent::Axis(event) = event {
                    events.push(event.clone());
                }
            }
            events
        };

        let mouse_motion = {
            if mouse_motion.delta != Vec2::ZERO {
                Some(mouse_motion.delta)
            } else {
                None
            }
        };

        let keycodes = Keycode {
            pressed: keyboard
                .get_pressed()
                .into_iter()
                .cloned()
                .collect::<HashSet<KeyCode>>(),
        };

        input_manager.set_motion(gamepad_axis_events, mouse_motion, keycodes);
    }
}
