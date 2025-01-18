use bevy::app::AppExit;
use bevy::{
    core_pipeline::core_2d::graph::input,
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use std::collections::HashMap;

fn main() {
    println!("Starting devy-bevy");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HelloEvent>()
            .insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Once)))
            .add_systems(
                Startup,
                (add_people, update_people, greet_people, greet_otherpeople).chain(),
            )
            .add_systems(
                Update,
                (hello_world, read_input, read_helloevent, check_greettimer),
            );
    }
}

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

fn hello_world() {
    // println!("hello bevy world");
}

#[derive(Component)]
struct Person;
#[derive(Component)]
struct OtherPerson;
#[derive(Component)]
struct Name(String);
fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
    commands.spawn((OtherPerson, Name("Adrian".to_string())));
}

enum HelloEventEnum {
    ExitEvent,
    OtherEvent,
    GeneralEvent,
}
#[derive(Event)]
struct HelloEvent(HelloEventEnum);

#[derive(Resource)]
struct GreetTimer(Timer);

fn read_input(
    time: Res<Time>,
    mut input_reader: EventReader<KeyboardInput>,
    mut ev: EventWriter<HelloEvent>,
    mut timer: ResMut<GreetTimer>,
) {
    for input in input_reader.read() {
        if !input.state.is_pressed() {
            continue;
        }
        match &input.key_code {
            KeyCode::Escape => {
                ev.send(HelloEvent(HelloEventEnum::ExitEvent));
            }
            KeyCode::KeyA => {
                timer.0.reset();
                println!("Setting timer at: {}", time.elapsed_secs());
            }
            _ => println!("input!!"),
        }
    }

    let mut mmap = HashMap::from([("adrian", 30), ("henrik", 29), ("alex", 28), ("magnus", 29)]);

    mmap.entry("stig").or_insert(24);

    let a = if mmap.contains_key("adian") {
        mmap.get("adian").unwrap()
    } else {
        &1
    };
    let b = &2;
    let c = if mmap.contains_key("ad") {
        mmap.get("adrian").unwrap()
    } else {
        &100
    };

    let d = {
        match mmap.get("henrik") {
            Some(t) => t,
            None => &66,
        }
    };
    let e = {
        match mmap.get("he") {
            Some(t) => t,
            None => &66,
        }
    };
    let f = {
        let mut t = 1;
        t += 1;
        t += 1;
        t += 1;
        t += 1;
        t
    };

    // println!("a: {}", a);
    // println!("b: {}", b);
    // println!("c: {}", c);
    // println!("d: {}", d);
    // println!("e: {}", e);
    // println!("f: {}", f);
}

fn check_greettimer(time: Res<Time>, mut timer: ResMut<GreetTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        println!(
            "HelloPlugin :: GreetTimer just_finished at: {}",
            time.elapsed_secs()
        );
    }
}

fn read_helloevent(mut er: EventReader<HelloEvent>, mut exit: EventWriter<AppExit>) {
    for event in er.read() {
        match event.0 {
            HelloEventEnum::ExitEvent => {
                exit.send(AppExit::Success);
            }
            _ => println!("reading other event"),
        }
    }
}

fn greet_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("hey there {}!", name.0)
    }
}
fn greet_otherpeople(query: Query<&Name, With<OtherPerson>>) {
    for name in &query {
        println!("You are someone else... {}!", name.0)
    }
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Adrian" {
            name.0 = "AdrianRS".to_string();
            break;
        }
    }
}
