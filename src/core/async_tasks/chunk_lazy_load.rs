use std::time::Instant;
use bevy::prelude::*;
use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem};
use crate::core::map::components::WorldChunk;

pub fn handle_background_tasks(
    task_system: ResMut<BackgroundTaskSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q: Query<(Entity, &mut Mesh3d, &mut WorldChunk)>,
) {
    let mut counter = 0;

    for result in task_system.receiver.try_iter() {
        match result {
            BackgroundTaskResult::ChunkLoaded(chunk_data) => {
                let timer = Instant::now();
                let mesh = meshes.add(chunk_data.mesh);
                
                if let Ok((_, mut mesh3d, _)) = q.get_mut(chunk_data.entity) {
                    mesh3d.0 = mesh;
                }

                counter = counter + 1;

                let remove_time = timer.elapsed();
                println!("chunk loaded {:?}", remove_time);
            },
            BackgroundTaskResult::ChunkGenerated(chunk_data) => {
                if let Ok((_, _, mut chunk)) = q.get_mut(chunk_data.entity) {
                    chunk.generated = true;
                    println!("chunk generated {:?}", chunk.id);
                }
            }
        }

        if counter >= 1 {
            return;
        }
    }
}