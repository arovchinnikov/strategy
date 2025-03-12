use std::time::Instant;
use bevy::prelude::*;
use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem};

pub fn handle_background_tasks(
    task_system: ResMut<BackgroundTaskSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Mesh3d)>
) {
    let mut counter = 0;

    for result in task_system.receiver.try_iter() {
        match result {
            BackgroundTaskResult::ChunkLoaded(chunk_data) => {
                let timer = Instant::now();
                let mesh = meshes.add(chunk_data.mesh);
                
                if let Ok((_, mut mesh3d)) = query.get_mut(chunk_data.entity) {
                    mesh3d.0 = mesh;
                }

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