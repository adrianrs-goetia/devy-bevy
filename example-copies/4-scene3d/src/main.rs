use bevy::{picking::pointer::PointerInteraction, prelude::*};
use exit_game::ExitGamePlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ExitGamePlugin, MeshPickingPlugin))
        .insert_resource(GroundEntity::default())
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .run();
}
#[derive(Component)]
struct Ground;

#[derive(Resource)]
struct GroundEntity {
    id: u32,
}
impl GroundEntity {
    fn default() -> GroundEntity {
        GroundEntity { id: 0 }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ground_entity: ResMut<GroundEntity>,
) {
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
