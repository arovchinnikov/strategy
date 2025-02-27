use bevy::app::{App, Plugin, Update};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::Startup;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::core::debug::framerate_screen::{counter_system, init_framerate_screen};

pub mod panic_handler;
mod framerate_screen;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Wireframes in all meshes
        app.add_plugins(WireframePlugin);

        app.add_plugins(WorldInspectorPlugin::new());

        // FPS debug view
        app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        app.add_systems(Startup, init_framerate_screen);
        app.add_systems(Update, counter_system);
    }
}
