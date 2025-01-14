use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::MouseMotion},
    prelude::*,
};
use std::sync::{Arc, Mutex};
pub struct InputManagerPlugin;

impl Plugin for InputManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Context::default())
            .add_systems(Update, read_input);
    }
}

fn read_input(
    commands: Commands,
    mut keyboard: EventReader<KeyboardInput>,
    // mouse: EventReader<MouseMotion>,
    // mouse: EventReader<MouseMotion>,
    // mouse_motion: Res<Motion<KeyCode>>,
    mut gamepad: EventReader<GamepadEvent>,
    // gamepad_axis: EventReader<>,
    input_context: Res<Context<'static>>,
) {
    let mut events: Vec<Binding> = vec![];
    // Sending gamepad input to context
    for ev in gamepad.read() {
        match ev {
            GamepadEvent::Axis(ev_axis) => {
                match ev_axis.axis {
                    GamepadAxis::LeftStickX => println!("{}", ev_axis.value),
                    _ => (),
                };
            }
            GamepadEvent::Button(ev_button) => {
                match ev_button.button {
                    GamepadButton::East => println!("{}", ev_button.state.is_pressed()),
                    _ => (),
                };
            }
            _ => (),
        }
    }
    for ev in keyboard.read() {
        if ev.state.is_pressed() {
            events.push(Binding::KeyBoard(ev.key_code));
        }
    }
    input_context_stack_handle_events(&events, &input_context);
}

fn input_context_stack_handle_events<'a>(
    input_events: &Vec<Binding>,
    input_context: &Context,
) {
    for event in input_events {
        input_context.default_context.handle_action(event);
    }
}

pub enum Binding {
    KeyBoard(KeyCode),
    MouseButton(MouseButton),
    MouseMotion,
    GamepadButton(GamepadButton),
    GamepadAxis(GamepadAxis),
}
pub struct SubContext {
    pub name: &'static str,
    pub blocking: bool,
    pub bindings: Vec<Binding>,
}

pub trait ActionBinding: Send + Sync {
    fn get_subcontext(&self) -> &SubContext;
    fn handle_action(&self, action: &Binding);
}

struct DefaultSubContext {
    context: SubContext,
}

impl ActionBinding for DefaultSubContext {
    fn get_subcontext(&self) -> &SubContext {
        &self.context
    }

    fn handle_action(&self, action: &Binding) {
        match action {
            Binding::KeyBoard(a) => {
                if let KeyCode::KeyA = a {
                    println!("KEY A")
                }
            }
            _ => (),
        }
    }
}

#[derive(Resource)]
struct Context<'a> {
    // default_context: SubContext,
    // default_context: Arc<Mutex<dyn ActionBinding + 'a>>,
    default_context: Box<dyn ActionBinding + 'a>,
    context_stack: Vec<Arc<Mutex<&'a SubContext>>>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            default_context: Box::new(DefaultSubContext {
                context: SubContext {
                    name: "DefaultSubContext",
                    blocking: true,
                    bindings: vec![Binding::KeyBoard(KeyCode::KeyA)],
                },
            }),
            context_stack: vec![],
        }
    }
}
