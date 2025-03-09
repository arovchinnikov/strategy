mod view_world;

use std::slice::Windows;
use bevy::app::{Startup, Update};
use bevy::math::{vec3, Vec3};
use bevy::prelude::{ButtonInput, Camera, Camera3d, Commands, Component, GlobalTransform, KeyCode, Query, Ray3d, Res, ResMut, Time, Transform, Vec2, Window};
use crate::core::map::camera::view_world::view_world;

#[derive(Component)]
struct CameraController {
    speed: f32,
}

#[derive(Component)]
struct CameraCorners {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

pub fn build(app: &mut bevy::prelude::App) {
    app.add_systems(Startup, init);
    app.add_systems(Update, camera_movement);
    app.add_systems(Update, update_camera_corners);
    app.add_systems(Update, view_world);
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1100.0, 180.0, 720.0).looking_at(vec3(1100.0, 0.0, 600.0), Vec3::Y),
        CameraController {
            speed: 440.0,
        },
        CameraCorners {
            min_x: 0.0,
            max_x: 0.0,
            min_z: 0.0,
            max_z: 0.0,
        }
    ));
}

fn update_camera_corners(
    window: Query<&Window>,
    mut query: Query<(&Camera, &GlobalTransform, &mut CameraCorners)>,
) {
    let window = window.single();
    let (camera, camera_transform, mut corners) = query.single_mut();

    let screen_corners = [
        Vec2::new(0.0, 0.0),
        Vec2::new(window.width(), 0.0),
        Vec2::new(0.0, window.height()),
    ];

    let mut world_corners = [Vec3::ZERO; 4];

    for (i, screen_corner) in screen_corners.iter().enumerate() {
        match camera.viewport_to_world(camera_transform, *screen_corner) {
            Ok(ray) => {
                if let Some(intersection) = ray_intersect_plane(ray, Vec3::Y, 0.0) {
                    world_corners[i] = intersection;
                }
            }
            Err(err) => {
                eprintln!("Ошибка преобразования координат: {:?}", err);
            }
        }
    }

    println!("{:?}", world_corners);

    corners.min_x = world_corners[0].x;
    corners.min_z = world_corners[0].z;
    corners.max_x = world_corners[1].x;
    corners.max_z = world_corners[2].z;
}

fn ray_intersect_plane(ray: Ray3d, plane_normal: Vec3, plane_d: f32) -> Option<Vec3> {
    let denom = ray.direction.dot(plane_normal);
    if denom.abs() > f32::EPSILON {
        let t = -(ray.origin.dot(plane_normal) + plane_d) / denom;
        if t >= 0.0 {
            return Some(ray.origin + t * ray.direction);
        }
    }
    None
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
