use std::time::Instant;
use bevy::prelude::*;
use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem};

pub fn handle_background_tasks(
    task_system: ResMut<BackgroundTaskSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut counter = 0;

    for result in task_system.receiver.try_iter() {
        match result {
            BackgroundTaskResult::ChunkLoaded(chunk_data) => {
                let timer = Instant::now();
                meshes.insert(chunk_data.mesh_id, chunk_data.mesh);

                counter = counter + 1;

                let remove_time = timer.elapsed();
                println!("chunk loaded {:?}", remove_time);
            }
        }

        if counter >= 1 {
            return;
        }
    }
}