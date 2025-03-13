use std::time::Instant;
use bevy::prelude::*;
use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem};
use crate::core::map::components::WorldChunk;
use crate::core::map::generator::mesh_pool::MeshPool;

pub fn handle_background_tasks(
    task_system: ResMut<BackgroundTaskSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_pool: ResMut<MeshPool>,
    mut q: Query<(Entity, &mut Mesh3d, &mut WorldChunk)>,
) {
    let mut counter = 0;

    for result in task_system.receiver.try_iter() {
        match result {
            BackgroundTaskResult::ChunkLoaded(chunk_data) => {
                let timer = Instant::now();

                if let Ok((entity, mut mesh3d, _)) = q.get_mut(chunk_data.entity) {
                    // Используем пул мешей вместо прямого создания
                    let mesh_handle = mesh_pool.get_mesh(entity, &mut meshes);

                    // Получаем меш из ресурса и заполняем его данными
                    if let Some(mesh) = meshes.get_mut(&mesh_handle) {
                        // Заполняем меш данными из загруженного чанка
                        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk_data.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().clone());
                        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, chunk_data.mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap().clone());
                        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk_data.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap().clone());
                        mesh.insert_indices(chunk_data.mesh.indices().unwrap().clone());
                    }

                    mesh3d.0 = mesh_handle;
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