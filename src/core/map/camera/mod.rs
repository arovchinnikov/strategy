use bevy::app::{Startup, Update};
use bevy::math::{vec3, Vec3};
use bevy::prelude::{ButtonInput, Camera3d, Commands, Component, KeyCode, Query, Res, Time, Transform};

#[derive(Component)]
struct CameraController {
    speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 440.0,
        }
    }
}

pub fn build(app: &mut bevy::prelude::App) {
    app.add_systems(Startup, init);
    app.add_systems(Update, camera_movement);
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1100.0, 340.0, 720.0).looking_at(vec3(1100.0, 0.0, 600.0), Vec3::Y),
        CameraController::default(),
    ));
}

fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&CameraController, &mut Transform)>,
) {
    for (controller, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * controller.speed * time.delta_secs();
        }
    }
}
