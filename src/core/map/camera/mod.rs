mod view_world;
mod zoom;

use crate::core::map::camera::view_world::{process_pending_mesh_deletions, view_world, PendingMeshDeletions};
use crate::core::map::camera::zoom::zoom_handler;
use bevy::app::{Startup, Update};
use bevy::math::Vec3;
use bevy::prelude::{ButtonInput, Camera, Camera3d, Commands, Component, FixedUpdate, GlobalTransform, KeyCode, MouseButton, Query, Ray3d, Res, ResMut, Resource, Time, Transform, Vec2, Window};

const MAP_MIN_X: f32 = -256.0;
const MAP_MAX_X: f32 = 8192.0 + 256.0;
const MAP_MIN_Z: f32 = -256.0;
const MAP_MAX_Z: f32 = 4096.0 + 256.0;

#[derive(Component)]
struct CameraController {
    speed: f32,
    zoom: CameraZoom
}

struct CameraZoom {
    speed: f32,
    target_height: f32,
    current_height: f32,
    smooth_factor: f32,
}

#[derive(Resource, Default)]
struct CameraDragState {
    is_dragging: bool,
    drag_start_world_position: Option<Vec3>,
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
    app.add_systems(Update, camera_drag_movement);
    app.add_systems(FixedUpdate, update_camera_corners);
    app.add_systems(Update, view_world);
    app.add_systems(FixedUpdate, zoom_handler);
    app.init_resource::<PendingMeshDeletions>();
    app.init_resource::<CameraDragState>();
    app.add_systems(Update, process_pending_mesh_deletions);
}

fn init(mut commands: Commands) {
    let initial_height = 120.0;

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1100.0, initial_height, 720.0),
        CameraController {
            speed: 440.0,
            zoom: CameraZoom {
                speed: 1200.0,
                target_height: initial_height,
                current_height: initial_height,
                smooth_factor: 0.1,
            }
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
            let new_position = transform.translation + direction * controller.speed * time.delta_secs();
            transform.translation = clamp_camera_position(new_position);
        }
    }
}

fn camera_drag_movement(
    window: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<CameraDragState>,
    mut query: Query<(&mut Transform, &GlobalTransform, &Camera)>,
) {
    let window = window.single();
    let (mut transform, global_transform, camera) = query.single_mut();

    if mouse_input.just_pressed(MouseButton::Right) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok(ray) = camera.viewport_to_world(global_transform, cursor_position) {
                if let Some(world_position) = ray_intersect_plane(ray, Vec3::Y, 0.0) {
                    drag_state.is_dragging = true;
                    drag_state.drag_start_world_position = Some(world_position);
                }
            }
        }
    }

    if mouse_input.just_released(MouseButton::Right) {
        drag_state.is_dragging = false;
        drag_state.drag_start_world_position = None;
    }

    if drag_state.is_dragging {
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(start_world_pos) = drag_state.drag_start_world_position {
                if let Ok(ray) = camera.viewport_to_world(global_transform, cursor_position) {
                    if let Some(current_world_pos) = ray_intersect_plane(ray, Vec3::Y, 0.0) {
                        let world_delta = start_world_pos - current_world_pos;

                        let movement = Vec3::new(world_delta.x, 0.0, world_delta.z);
                        let new_position = transform.translation + movement;
                        transform.translation = clamp_camera_position(new_position);
                    }
                }
            }
        }
    }
}

fn get_cursor_world_delta(
    cursor_delta: Vec2,
    global_transform: &GlobalTransform,
) -> Vec3 {
    let delta = Vec2::new(cursor_delta.x, -cursor_delta.y);

    let right = global_transform.right();
    let forward = global_transform.forward();

    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize();

    right_xz * delta.x + forward_xz * delta.y
}

fn clamp_camera_position(position: Vec3) -> Vec3 {
    Vec3::new(
        position.x.clamp(MAP_MIN_X, MAP_MAX_X),
        position.y,
        position.z.clamp(MAP_MIN_Z, MAP_MAX_Z),
    )
}
