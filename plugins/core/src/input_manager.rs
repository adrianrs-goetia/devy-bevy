use bevy::input::gamepad::GamepadEvent;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use std::collections::HashMap;

pub struct InputManagerPlugin;
impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputManager::default()).add_systems(
            PreUpdate,
            (
                determine_input_mode,
                button::read_button_input,
                motion::read_motion_input,
            ),
        );
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Action(pub &'static str);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputType {
    Keyboard,
    Mouse,
    Gamepad,
}

impl InputType {
    fn is_mode(&self, mode: InputMode) -> bool {
        match mode {
            InputMode::MouseAndKeyboard => *self == Self::Keyboard || *self == Self::Mouse,
            InputMode::Gamepad => *self == Self::Gamepad,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    MouseAndKeyboard,
    Gamepad, // xbox gamepad assumed
}

#[derive(Debug, Clone, Component)]
#[allow(dead_code)]
pub struct InputModeChanged(InputMode);

impl Event for InputModeChanged {
    // https://bevyengine.org/examples/ecs-entity-component-system/observer-propagation/
    type Traversal = &'static Parent;
    const AUTO_PROPAGATE: bool = true;
}

// determine input mode an observable trigger on change
fn determine_input_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    gamepad_events: EventReader<GamepadEvent>,

    mut input_manager: ResMut<InputManager>,
    mut commands: Commands,
) {
    let mut input_mode: Option<InputMode> = None;

    if keyboard.get_pressed().len() != 0
        || mouse.get_pressed().len() != 0
        || mouse_motion.delta != Vec2::ZERO
    {
        input_mode = Some(InputMode::MouseAndKeyboard);
    }

    // gamepad mode takes priority over MnK
    if gamepad_events.len() != 0 {
        input_mode = Some(InputMode::Gamepad);
    }

    if let Some(input_mode) = input_mode {
        if input_manager.change_input_mode(input_mode) {
            commands.trigger(InputModeChanged(input_mode));
        }
    }
}

#[derive(Resource)]
pub struct InputManager {
    current_input_mode: InputMode,
    button_entries: HashMap<Action, button::ActionEntry>,
    motion_entries: HashMap<Action, motion::ActionEntry>,
}

impl InputManager {
    pub(crate) fn change_input_mode(&mut self, new_mode: InputMode) -> bool {
        if !self.current_input_mode.eq(&new_mode) {
            self.current_input_mode = new_mode;
            return true;
        }
        false
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            current_input_mode: InputMode::MouseAndKeyboard,
            button_entries: HashMap::<Action, button::ActionEntry>::new(),
            motion_entries: HashMap::<Action, motion::ActionEntry>::new(),
        }
    }
}

impl InputManager {
    /**
     * Motions are applied in the order they come in, and later motion entries
     * will overwrite previous ones
     */
    pub fn register_action_motion(&mut self, action: Action, entries: Vec<motion::Entry>) {
        self.motion_entries.insert(
            action,
            motion::ActionEntry {
                motion_entries: entries
                    .iter()
                    .map(|e| (e.clone(), false)) // Disgusting clone
                    .into_iter()
                    .collect(),
                motion: Vec2::new(0., 0.),
            },
        );
    }

    pub fn get_motion(&self, action: Action) -> Vec2 {
        if let Some(entry) = self.motion_entries.get(&action) {
            if entry.motion.length() > 1.0 {
                if let Some(normal) = entry.motion.try_normalize() {
                    return normal;
                }
            }
            return entry.motion;
        }
        unreachable!("Missing action: {}", action.0)
    }

    pub fn get_motion3z(&self, action: Action) -> Vec3 {
        let v2 = self.get_motion(action);
        Vec3 {
            x: -v2.x, // TODO how to handle x-axis in 3d space
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

    pub fn register_action_button(&mut self, action: Action, buttons: Vec<button::Variant>) {
        self.button_entries.insert(
            action,
            button::ActionEntry {
                just_pressed: HashSet::<button::Variant>::new(),
                pressed: HashSet::<button::Variant>::new(),
                just_released: HashSet::<button::Variant>::new(),
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

    fn set_button_pressed(&mut self, button: button::Variant) {
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

    fn set_button_released(&mut self, button: button::Variant) {
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

pub mod button {
    use bevy::{
        input::{gamepad::GamepadEvent, mouse::MouseButtonInput, ButtonState},
        prelude::*,
        utils::HashSet,
    };

    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum Variant {
        Keyboard(KeyCode),
        Mouse(MouseButton),
        Gamepad(GamepadButton),
    }

    pub(super) struct ActionEntry {
        pub just_pressed: HashSet<Variant>,
        pub pressed: HashSet<Variant>,
        pub just_released: HashSet<Variant>,
        pub released: HashSet<Variant>,
    }

    pub(super) fn read_button_input(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut mouse: EventReader<MouseButtonInput>,
        mut gamepad: EventReader<GamepadEvent>,
        mut input_manager: ResMut<super::InputManager>,
    ) {
        input_manager.move_prev_frame_just_pressed();
        input_manager.move_prev_frame_just_released();

        for key in keyboard.get_just_pressed() {
            input_manager.set_button_pressed(Variant::Keyboard(*key));
        }
        for key in keyboard.get_just_released() {
            input_manager.set_button_released(Variant::Keyboard(*key));
        }

        for event in mouse.read() {
            match event.state {
                ButtonState::Pressed => {
                    input_manager.set_button_pressed(Variant::Mouse(event.button))
                }
                ButtonState::Released => {
                    input_manager.set_button_released(Variant::Mouse(event.button))
                }
            }
        }

        for event in gamepad.read() {
            match event {
                GamepadEvent::Button(button) => {
                    if button.state.is_pressed() {
                        input_manager.set_button_pressed(Variant::Gamepad(button.button));
                    } else {
                        input_manager.set_button_released(Variant::Gamepad(button.button));
                    }
                }
                _ => (),
            }
        }
    }
}

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

    #[derive(Clone)]
    pub enum Relation {
        GamepadAxis(GamepadAxis, Axis),
        Mouse(
            // acts as sensitivity, mouse motion is dividied by this
            f32,
        ),
        KeyCode(KeyCode, Axis),
    }

    pub(super) struct KeyCodeSet {
        pressed: HashSet<KeyCode>,
        released: HashSet<KeyCode>,
    }

    impl KeyCodeSet {
        fn is_key_pressed(&self, key: KeyCode) -> bool {
            return self.pressed.contains(&key);
        }

        fn is_key_released(&self, key: KeyCode) -> bool {
            return self.released.contains(&key);
        }

        fn is_empty(&self) -> bool {
            return self.pressed.is_empty() && self.released.is_empty();
        }
    }

    #[derive(Clone)]
    pub struct Entry {
        pub input_type: super::InputType,
        pub relations: Vec<Relation>,
    }

    pub struct ActionEntry {
        /**
         * bool -- motion_last_frame
         * If there was motion last frame and none this frame,
         * then the entry might want to overwrite the motion vector to Vec2::ZERO
         */
        pub motion_entries: Vec<(Entry, bool)>,
        pub motion: Vec2,
    }

    impl ActionEntry {
        pub(super) fn set_motion(
            &mut self,
            input_mode_priority: super::InputMode,
            axis_events: &Vec<GamepadAxisChangedEvent>,
            mouse_motion: &Option<Vec2>,
            keyboard: &KeyCodeSet,
        ) {
            for (mapping, motion_last_frame) in self
                .motion_entries
                .iter_mut()
                .filter(|(m, _)| m.input_type.is_mode(input_mode_priority))
            {
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
                        Self::set_mouse_motion(
                            &mut self.motion,
                            *motion_last_frame,
                            &mapping.relations,
                            mouse_motion,
                        );
                        *motion_last_frame = mouse_motion.is_some();
                    }
                };
            }
        }

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
            motion: &mut Vec2,
            relations: &Vec<Relation>,
            pressed_keycodes: &KeyCodeSet,
        ) {
            if pressed_keycodes.is_empty() {
                return;
            }

            let mut new_motion = Vec2::ZERO;
            for relation in relations {
                if let Relation::KeyCode(keycode, axis) = relation {
                    if pressed_keycodes.is_key_pressed(*keycode) {
                        new_motion += axis.get_value_v2()
                    }
                    if pressed_keycodes.is_key_released(*keycode) {
                        match axis {
                            Axis::PosY | Axis::NegY => new_motion.y = 0.0,
                            Axis::PosX | Axis::NegX => new_motion.x = 0.0,
                            _ => (),
                        }
                    }
                }
            }
            *motion = new_motion;
        }

        fn set_mouse_motion(
            motion: &mut Vec2,
            motion_last_frame: bool,
            relations: &Vec<Relation>,
            mouse_motion: &Option<Vec2>,
        ) {
            assert!(relations.len() == 1);
            if let Relation::Mouse(normalizing_factor) = relations[0] {
                if let Some(mouse_motion) = mouse_motion {
                    *motion = mouse_motion / normalizing_factor
                } else if motion_last_frame {
                    *motion = Vec2::ZERO
                }
            }
        }
    }

    pub(super) fn read_motion_input(
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
            released: keyboard
                .get_just_released()
                .into_iter()
                .cloned()
                .collect::<HashSet<KeyCode>>(),
        };

        let mode = input_manager.current_input_mode;
        for action_entry in input_manager.motion_entries.values_mut() {
            action_entry.set_motion(mode, &gamepad_axis_events, &mouse_motion, &keycodes);
        }
    }
}
