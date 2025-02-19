use bevy::app::Update;
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, Res};

pub mod panic_handler;

pub fn init(app: &mut bevy::prelude::App) {
    app.add_plugins(bevy::prelude::DefaultPlugins);

    app.add_systems(Update, trigger_panic);
}

fn trigger_panic(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyP) {
        panic!("Manual panic triggered!");
    }
}
