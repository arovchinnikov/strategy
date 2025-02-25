use bevy::app::Update;
use crate::core::map::MapPlugin;
use debug::panic_handler::trigger_panic;
use crate::core::debug::DebugPlugin;

mod map;
pub(crate) mod debug;

pub fn init(app: &mut bevy::prelude::App) {
    app.add_plugins(bevy::prelude::DefaultPlugins);
    app.add_plugins(MapPlugin);
    app.add_plugins(DebugPlugin);

    app.add_systems(Update, trigger_panic);
}
