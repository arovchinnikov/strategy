use crate::core::map::camera::{CameraController, CameraLodState};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::{EulerRot, EventReader, Quat, Query, Res, ResMut, Time, Transform};

const MIN_HEIGHT: f32 = 60.0;
const MAX_HEIGHT: f32 = 1300.0;
const MIN_TILT: f32 = -0.6;
const MAX_TILT: f32 = -1.35;

pub fn zoom_handler(
    time: Res<Time>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&mut CameraController, &mut Transform)>,
    mut lod_state: ResMut<CameraLodState>,
) {
    let mut scroll = 0.0;
    for event in mouse_wheel_events.read() {
        scroll -= event.y;
    }

    for (mut controller, mut transform) in query.iter_mut() {
        controller.zoom.target_height -= scroll * controller.zoom.speed * time.delta_secs();
        controller.zoom.target_height = controller.zoom.target_height.clamp(MIN_HEIGHT, MAX_HEIGHT);

        if controller.zoom.target_height == controller.zoom.current_height {
            continue;
        }

        controller.zoom.current_height = lerp(
            controller.zoom.current_height,
            controller.zoom.target_height,
            controller.zoom.smooth_factor
        );

        transform.translation.y = controller.zoom.current_height;
        let pitch_angle = height_to_tilt(controller.zoom.current_height);
        let (yaw, _, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch_angle, roll);
        lod_state.current_height = controller.zoom.current_height;
    }
}


fn height_to_tilt(height: f32) -> f32 {
    let clamped_height = height.clamp(MIN_HEIGHT, MAX_HEIGHT);
    let t = (clamped_height - MIN_HEIGHT) / (MAX_HEIGHT - MIN_HEIGHT);
    let transformed_t = 2.0 * t - t * t;
    MIN_TILT + (MAX_TILT - MIN_TILT) * transformed_t
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}
