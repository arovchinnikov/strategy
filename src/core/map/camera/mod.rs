mod view_world;

use crate::core::map::camera::view_world::{process_pending_mesh_deletions, view_world, PendingMeshDeletions};
use bevy::app::{Startup, Update};
use bevy::math::{vec3, Vec3};
use bevy::prelude::{ButtonInput, Camera, Camera3d, Commands, Component, FixedUpdate, GlobalTransform, KeyCode, MouseButton, Query, Ray3d, Res, ResMut, Resource, Time, Transform, Vec2, Window};

const MAP_MIN_X: f32 = -256.0;
const MAP_MAX_X: f32 = 8192.0 + 256.0;
const MAP_MIN_Z: f32 = -256.0;
const MAP_MAX_Z: f32 = 4096.0 + 256.0;

#[derive(Component)]
struct CameraController {
    speed: f32,
    drag_speed: f32,
}

#[derive(Resource, Default)]
struct CameraDragState {
    is_dragging: bool,
    last_cursor_position: Option<Vec2>,
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
    app.init_resource::<PendingMeshDeletions>();
    app.init_resource::<CameraDragState>();
    app.add_systems(Update, process_pending_mesh_deletions);
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1100.0, 180.0, 720.0).looking_at(vec3(1100.0, 0.0, 600.0), Vec3::Y),
        CameraController {
            speed: 440.0,
            drag_speed: 40.5,
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
    time: Res<Time>,
    window: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<CameraDragState>,
    mut query: Query<(&CameraController, &mut Transform, &GlobalTransform)>,
) {
    let window = window.single();
    let (controller, mut transform, global_transform) = query.single_mut();

    if mouse_input.just_pressed(MouseButton::Right) {
        drag_state.is_dragging = true;
        drag_state.last_cursor_position = window.cursor_position();
    }

    if mouse_input.just_released(MouseButton::Right) {
        drag_state.is_dragging = false;
        drag_state.last_cursor_position = None;
    }

    if drag_state.is_dragging {
        if let Some(current_cursor) = window.cursor_position() {
            if let Some(last_cursor) = drag_state.last_cursor_position {
                let delta = current_cursor - last_cursor;

                if delta.length_squared() > 0.0 {
                    let cursor_delta_world = get_cursor_world_delta(delta, global_transform) * -1.0;

                    let movement = cursor_delta_world * controller.drag_speed * time.delta_secs();
                    let new_position = transform.translation + movement;
                    transform.translation = clamp_camera_position(new_position);
                }
            }

            drag_state.last_cursor_position = Some(current_cursor);
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
