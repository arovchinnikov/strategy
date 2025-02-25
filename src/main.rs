#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod core;

use std::panic;
use bevy::prelude::*;
use self::core::debug::panic_handler::setup_panic_handler;

fn main() {
    setup_panic_handler();
    let result = panic::catch_unwind(|| {
        let mut app = App::new();
        core::init(&mut app);

        app.run();
    });

    if let Err(err) = result {
        // TODO GUI with crush report
        eprintln!("Crush: {:?}", err);
    }
}
