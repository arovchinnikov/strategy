mod camera;
mod sea;
mod light;
pub(crate) mod terrain;
pub(crate) mod components;

use crate::core::map::terrain::generate_terrain;
use bevy::app::{App, Plugin, Startup};

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        terrain::build(app);
        camera::build(app);
        sea::build(app);
        light::build(app);
        terrain::mesh_pool::build(app);

        app.add_systems(Startup, generate_terrain);
    }
}
