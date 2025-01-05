use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};

/**
 * Copy paste from examples, hell yeee
 */

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        #[cfg(not(target_arch = "wasm32"))]
        Wireframe2dPlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, toggle_wireframe);
    app.run();
}

const X_EXTENT: f32 = 900.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    commands.spawn(Camera2d);

    let shapes = [
        meshes.add(Circle::new(50.0)),
        meshes.add(CircularSector::new(60.0, 1.0)),
        meshes.add(CircularSegment::new(70.0, 1.25)),
        meshes.add(Ellipse::new(25.0, 40.0)),
        meshes.add(Circle::new(50.0)),
        // meshes.add(Circle::new(50.0)),
        // meshes.add(Circle::new(50.0)),
        // meshes.add(Circle::new(50.0)),
    ];
    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate(){
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT, 0.0, 0.0)
        ));
    }

    commands.spawn((
        Text::new("Press space"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn toggle_wireframe (
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
){
    if keyboard.just_pressed(KeyCode::Space){
        wireframe_config.global = !wireframe_config.global;
    }
}