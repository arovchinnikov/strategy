use bevy::app::Startup;
use bevy::color::palettes::basic::WHITE;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{DirectionalLight, DirectionalLightShadowMap};
use bevy::prelude::{default, Commands, ResMut, Transform};
use std::f32::consts::PI;

pub fn build(app: &mut bevy::prelude::App) {
    app.add_systems(Startup, init);
}

pub fn init(
    mut commands: Commands,
    mut shadow_settings: ResMut<DirectionalLightShadowMap>,
) {
    shadow_settings.size = 4096;

    commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadows_enabled: true,
            shadow_depth_bias: 0.002,
            ..default()
        },
        Transform::from_xyz(0.0, 2000.0, 0.0).with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.))
    ));
}
