use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem};
use crate::core::async_tasks::chunk_loading::process_loaded_chunk;
use crate::core::map::components::WorldChunk;
use crate::core::map::terrain::mesh_pool::MeshPool;

pub fn handle_background_tasks(
    task_system: ResMut<BackgroundTaskSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_pool: ResMut<MeshPool>,
    mut q: Query<(Entity, &mut Mesh3d, &mut WorldChunk)>,
) {
    let max_tasks_per_frame = 4;
    let mut processed_tasks = 0;

    for result in task_system.receiver.try_iter() {
        match result {
            BackgroundTaskResult::ChunkLoaded(chunk_data) => {
                if process_loaded_chunk(chunk_data, &mut meshes, &mut mesh_pool, &mut q) {
                    processed_tasks += 1;
                    if processed_tasks >= max_tasks_per_frame {
                        return;
                    }
                }
            },
            BackgroundTaskResult::ChunkGenerated(chunk_data) => {
                if let Ok((_, _, mut chunk)) = q.get_mut(chunk_data.entity) {
                    chunk.generated = true;
                }
            }
        }
    }
}
