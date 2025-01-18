use bevy::input::gamepad::GamepadAxisChangedEvent;
use bevy::input::mouse::MouseButtonInput;
use bevy::utils::hashbrown::HashSet;
use bevy::{
    input::{gamepad::GamepadEvent, ButtonState},
    prelude::*,
};
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
            motion_entries: HashMap::<Action, motion::ActionEntry>::new(),
        }
    }
}

impl InputManager {
    /**
     * Motions are applied in the order they come in, and later motion entries
     * will overwrite previous ones
     */
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
        keyboard: motion::KeyCodeSet,
    ) {
        for action_entry in self.motion_entries.values_mut() {
            action_entry.set_motion(&axis_events, &mouse_motion, &keyboard);
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

    #[derive(Clone, Copy)]
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

        pub fn get_value_v2(&self) -> Vec2 {
            match self {
                Self::PosX => Vec2::X,
                Self::NegX => Vec2::NEG_X,
                Self::PosY => Vec2::Y,
                Self::NegY => Vec2::NEG_Y,
                _ => Vec2::ZERO,
            }
        }
    }

    pub enum Relation {
        GamepadAxis(GamepadAxis, Axis),
        Mouse(
            // acts as sensitivity,
            // mouse motion is dividied by this
            f32,
        ),
        KeyCode(KeyCode, Axis),
    }

    pub(super) struct KeyCodeSet {
        pressed: HashSet<KeyCode>,
    }

    impl KeyCodeSet {
        pub(super) fn is_key_pressed(&self, key: KeyCode) -> bool {
            return self.pressed.contains(&key);
        }
    }

    pub struct Entry {
        pub input_type: super::InputType,
        pub relations: Vec<Relation>,
    }

    pub struct ActionEntry {
        pub motion_entries: Vec<Entry>,
        pub motion: Vec2,
    }

    impl ActionEntry {
        pub(super) fn set_motion(
            &mut self,
            axis_events: &Vec<GamepadAxisChangedEvent>,
            mouse_motion: &Option<Vec2>,
            keyboard: &KeyCodeSet,
        ) {
            for mapping in self.motion_entries.iter() {
                match mapping.input_type {
                    super::InputType::Gamepad => Self::set_gamepad_axis_motion(
                        &mut self.motion,
                        &mapping.relations,
                        axis_events,
                    ),
                    super::InputType::Keyboard => {
                        Self::set_keyboard_motion(&mut self.motion, &mapping.relations, keyboard)
                    }
                    super::InputType::Mouse => {
                        Self::set_mouse_motion(&mut self.motion, &mapping.relations, mouse_motion)
                    }
                };
            }
        }

        // fn set_gamepad_axis_motion(&mut self, relations: &Vec<Relation>, axis_events: &Vec<GamepadAxisChangedEvent>){
        fn set_gamepad_axis_motion(
            motion: &mut Vec2,
            relations: &Vec<Relation>,
            axis_events: &Vec<GamepadAxisChangedEvent>,
        ) {
            for relation in relations {
                if let Relation::GamepadAxis(relation_gamepad_axis, relation_axis) = relation {
                    for gamepad_event in axis_events
                        .iter()
                        .filter(|a| a.axis == *relation_gamepad_axis)
                    {
                        match relation_axis {
                            Axis::X => motion.x = gamepad_event.value,
                            Axis::Y => motion.y = gamepad_event.value,
                            _ => (),
                        }
                    }
                }
            }
        }

        fn set_keyboard_motion(
            // &mut self,
            motion: &mut Vec2,
            relations: &Vec<Relation>,
            pressed_keycodes: &KeyCodeSet,
        ) {
            let mut new_motion = Vec2::ZERO;
            for relation in relations {
                if let Relation::KeyCode(keycode, axis) = relation {
                    if pressed_keycodes.is_key_pressed(*keycode) {
                        new_motion += axis.get_value_v2()
                    }
                }
            }
            *motion = new_motion;
        }

        fn set_mouse_motion(
            // &mut self,
            motion: &mut Vec2,
            relations: &Vec<Relation>,
            mouse_motion: &Option<Vec2>,
        ) {
            assert!(relations.len() == 1);
            if let Relation::Mouse(normalizing_factor) = relations[0] {
                if let Some(mouse_motion) = mouse_motion {
                    *motion = mouse_motion / normalizing_factor;
                }
            }
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

        let keycodes = KeyCodeSet {
            pressed: keyboard
                .get_pressed()
                .into_iter()
                .cloned()
                .collect::<HashSet<KeyCode>>(),
        };

        input_manager.set_motion(gamepad_axis_events, mouse_motion, keycodes);
    }
}
