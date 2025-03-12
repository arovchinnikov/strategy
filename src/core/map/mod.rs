mod camera;
mod sea;
mod light;
mod generator;
mod components;

use crate::core::map::generator::generate_terrain;
use bevy::app::{App, Plugin, Startup};
use bevy::prelude::{BuildChildren, Component};
use std::io::{Read, Write};

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        generator::build(app);
        camera::build(app);
        sea::build(app);
        light::build(app);
        
        app.add_systems(Startup, generate_terrain);
    }
}
