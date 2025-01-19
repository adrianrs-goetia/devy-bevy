use std::{f32::consts::PI, time::Duration};

// use bevy::core::FrameCount;
use bevy::{picking::pointer::PointerInteraction, prelude::*};

use core::exit_game::ExitGamePlugin;
use core::input_manager::{
    button, motion, Action, InputManager, InputManagerPlugin, InputModeChanged,
};

const BOXY_PATH: &str = "models/boxy.glb";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ExitGamePlugin,
            InputManagerPlugin,
            MeshPickingPlugin,
        ))
        .insert_resource(GroundEntity::default())
        .add_systems(Startup, (setup, register_input))
        .add_systems(Update, (setup_once_loaded, read_im_input))
        .add_systems(
            Update,
            (draw_cursor, rotate_boxy, keyboard_animation_control),
        )
        .add_observer(get_input_mode_change_trigger)
        .run();
}

static LEFT: Action = Action("left");
static MOVEMENT: Action = Action("movement");

fn register_input(mut im: ResMut<InputManager>) {
    im.register_button_events(
        LEFT,
        vec![
            button::Variant::Keyboard(KeyCode::KeyA),
            button::Variant::Mouse(MouseButton::Left),
            button::Variant::Keyboard(KeyCode::ArrowLeft),
            button::Variant::Gamepad(GamepadButton::East),
        ],
    );

    im.register_motion(
        MOVEMENT,
        vec![
            motion::Entry {
                input_type: core::input_manager::InputType::Keyboard,
                relations: vec![
                    motion::Relation::KeyCode(KeyCode::KeyW, motion::Axis::PosY),
                    motion::Relation::KeyCode(KeyCode::KeyS, motion::Axis::NegY),
                    motion::Relation::KeyCode(KeyCode::KeyD, motion::Axis::PosX),
                    motion::Relation::KeyCode(KeyCode::KeyA, motion::Axis::NegX),
                ],
            },
            motion::Entry {
                input_type: core::input_manager::InputType::Mouse,
                relations: vec![motion::Relation::Mouse(20.)],
            },
            motion::Entry {
                input_type: core::input_manager::InputType::Gamepad,
                relations: vec![
                    motion::Relation::GamepadAxis(GamepadAxis::LeftStickY, motion::Axis::Y),
                    motion::Relation::GamepadAxis(GamepadAxis::LeftStickX, motion::Axis::X),
                ],
            },
        ],
    );
}

fn get_input_mode_change_trigger(trigger: Trigger<InputModeChanged>) {
    let event = trigger.event();
    println!("TRIGGER input_mode_change: {:?}", event);
}

fn read_im_input(
    im: Res<InputManager>,
    // fc: Res<FrameCount>
) {
    if im.is_action_just_pressed(LEFT) {
        println!(" !!! LEFT IS JUST PRESSED !!! ")
    }
    if im.is_action_just_released(LEFT) {
        println!(" LEFT REALEASED ")
    }

    // let motion = im.get_motion(MOVEMENT);
    // println!("{}, motion: {}", fc.0, motion);
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Boxy;

#[derive(Resource)]
struct GroundEntity {
    id: u32,
}
impl GroundEntity {
    fn default() -> GroundEntity {
        GroundEntity { id: 0 }
    }
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ground_entity: ResMut<GroundEntity>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // boxy
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(BOXY_PATH)),
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(BOXY_PATH)),
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph: graph_handle,
    });
    let boxy_transform: Transform = {
        let mut transform = Transform::from_xyz(0.0, 1.0, 0.0);
        transform.rotate_y(PI * 1.5);
        transform
    };
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(BOXY_PATH))),
        boxy_transform,
        Boxy,
    ));

    // ground
    ground_entity.id = commands
        .spawn((
            Mesh3d(meshes.add(Circle::new(4.0))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            Ground,
        ))
        .id()
        .index();

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(120, 0, 180))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        // PickingBehavior::IGNORE,
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    // println!("AnimationPlayer loaded...");
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}

fn draw_cursor(
    pointers: Query<&PointerInteraction>,
    mut gizmos: Gizmos,
    ground_entity: Res<GroundEntity>,
) {
    // draw circle just above ground plane
    for (entity, point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(entity, hit)| {
            hit.position
                .zip(hit.normal)
                .map(|(position, normal)| (entity, position, normal))
        })
    {
        if entity.index() == ground_entity.id {
            gizmos.circle(
                Isometry3d::new(
                    point + normal * 0.01,
                    Quat::from_rotation_arc(Vec3::Z, normal),
                ),
                0.2,
                Color::WHITE,
            );
        }
    }
}

fn rotate_boxy(_time: Res<Time>, _boxy: Single<&mut Transform, With<Boxy>>) {
    // boxy.rotate_y(0.2 * TAU * time.delta_secs());
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for (mut player, mut transitions) in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Enter) {
            *current_animation = (*current_animation + 1) % animations.animations.len();

            transitions
                .play(
                    &mut player,
                    animations.animations[*current_animation],
                    Duration::ZERO,
                )
                .repeat();
        }
    }
}
