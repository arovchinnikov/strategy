use std::panic;
use backtrace::Backtrace;
use std::fs;
use bevy::prelude::{ButtonInput, KeyCode, Res};
use chrono::Local;

pub fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let backtrace = Backtrace::new();
        let report = format!(
            "Crash Report:\n\n{}\nBacktrace:\n{:?}",
            panic_info,
            backtrace
        );

        let log_dir = "logs";
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Failed to create logs directory: {}", e);
            return;
        }

        let timestamp = Local::now().format("%Y%m%d%H%M%S");
        let filename = format!("{}/crash_report_{}.txt", log_dir, timestamp);

        if let Err(e) = fs::write(&filename, &report) {
            eprintln!("Failed to write crash report: {}", e);
        } else {
            eprintln!("Crash report saved to: {}", filename);
        }
    }));
}

pub fn trigger_panic(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyP) {
        panic!("Manual panic triggered!");
    }
}
