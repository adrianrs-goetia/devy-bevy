use bevy::{
    asset::io::memory::Dir, prelude::*, text::cosmic_text::ttf_parser::cff::Matrix, utils::HashMap,
};

const UP: Dir3 = Dir3::Y;

pub struct IsometricCameraPlugin;
impl Plugin for IsometricCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraManager>()
            .add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum CameraMode {
    Game,
    Editor,
}

#[derive(Debug, Resource)]
pub struct CameraManager {
    current_mode: CameraMode,
    cameras: HashMap<CameraMode, IsometricCamera>,
}

impl Default for CameraManager {
    fn default() -> Self {
        let default_mode = CameraMode::Game;
        Self {
            current_mode: default_mode,
            cameras: HashMap::from([
                (default_mode, IsometricCamera::default()),
                (CameraMode::Editor, IsometricCamera::default()),
            ]),
        }
    }
}

impl CameraManager {
    fn get_camera_transform(&self) -> Transform {
        self.get().get_camera_transform()
    }

    fn get(&self) -> &IsometricCamera {
        self.cameras.get(&self.current_mode).unwrap()
    }

    fn get_mut(&mut self) -> &mut IsometricCamera {
        self.cameras.get_mut(&self.current_mode).unwrap()
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        assert!(self.cameras.contains_key(&mode));
        self.current_mode = mode
    }

    pub fn move_camera(&mut self, movement: Vec3) {
        self.get_mut().move_camera(movement)
    }
}

#[derive(Debug, Component)]
pub struct IsometricCamera {
    pivot: Vec3,
    angle_yaw: f32,
    angle_pitch: f32,
    spring_arm_length: f32,
}

impl Default for IsometricCamera {
    fn default() -> Self {
        Self {
            pivot: Vec3::ZERO,
            angle_yaw: 45., // angles in degrees
            angle_pitch: -45.,
            // angle_yaw: 0., // angles in degrees
            // angle_pitch: 0.,
            spring_arm_length: 20.,
        }
    }
}

impl IsometricCamera {
    fn move_camera(&mut self, movement: Vec3) {
        self.pivot += movement
    }

    fn get_camera_transform(&self) -> Transform {
        let mut transform = Transform::from_translation(self.pivot);
        transform.rotate_y(self.angle_yaw.to_radians());
        let axis = {
            let forward = transform.forward();
            if let Ok(dir) = Dir3::new(UP.cross(forward.as_vec3()).normalize()) {
                dir
            } else {
                Dir3::X
            }
        };
        transform.rotate_axis(axis, self.angle_pitch.to_radians());

        let pos = self.pivot + (transform.forward() * self.spring_arm_length);

        transform = transform.with_translation(pos);
        transform.looking_at(self.pivot, UP)

        // Transform::from_translation(Vec3::X + Vec3::Y).looking_at(Vec3::ZERO, UP)
    }
}

fn setup(mut commands: Commands, camera_manager: Res<CameraManager>) {
    commands.spawn((Camera3d::default(), camera_manager.get_camera_transform()));
    println!(
        "camera transform: {:?}",
        camera_manager.get_camera_transform()
    );
}

fn update(camera_manager: Res<CameraManager>, mut transform: Single<&mut Transform, With<Camera>>) {
    **transform = camera_manager.get_camera_transform();
}
