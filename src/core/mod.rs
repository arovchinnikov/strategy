use crate::core::debug::DebugPlugin;
use crate::core::map::MapPlugin;
use bevy::app::Update;
use bevy::DefaultPlugins;
use bevy::prelude::{default, AssetPlugin, AssetServer, Commands, PluginGroup, Res, Startup};
use bevy_audio::{AudioPlayer, PlaybackSettings};
use debug::panic_handler::trigger_panic;

mod map;
pub(crate) mod debug;

pub fn init(app: &mut bevy::prelude::App) {
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: "common".to_string(),
        ..Default::default()
    }));
    app.add_plugins(MapPlugin);
    app.add_plugins(DebugPlugin);

    app.add_systems(Update, trigger_panic);
    app.add_systems(Startup, setup);
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    let music_handle = asset_server.load("sound/music/carved-in-stone.ogg");
    commands.spawn((
        AudioPlayer::new(music_handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.5),
            ..default()
        },
    ));
}
