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
    // Увеличиваем лимит обработки задач
    let max_tasks_per_frame = 4;
    let mut processed_tasks = 0;

    for result in task_system.receiver.try_iter() {
        match result {
            BackgroundTaskResult::ChunkLoaded(chunk_data) => {
                let timer = Instant::now();
                let lod_level = chunk_data.lod.unwrap_or_else(|| {
                    crate::core::map::generator::cache::LodLevel::High
                });

                if let Ok((entity, mut mesh3d, mut chunk)) = q.get_mut(chunk_data.entity) {
                    // Проверяем, нужен ли все еще этот LOD и загружен ли чанк
                    if !chunk.loaded || chunk.target_lod != Some(lod_level) {
                        #[cfg(debug_assertions)]
                        println!("Пропуск загрузки меша для чанка {} с LOD {:?}, т.к. чанк не загружен или нужен другой LOD",
                                 chunk.id, lod_level);
                        continue;
                    }

                    let mesh_data = {
                        let positions = match chunk_data.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
                            bevy::render::mesh::VertexAttributeValues::Float32x3(values) => values.clone(),
                            _ => panic!("Неожиданный формат для positions"),
                        };

                        let normals = match chunk_data.mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
                            bevy::render::mesh::VertexAttributeValues::Float32x3(values) => values.clone(),
                            _ => panic!("Неожиданный формат для normals"),
                        };

                        let uvs = match chunk_data.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() {
                            bevy::render::mesh::VertexAttributeValues::Float32x2(values) => values.clone(),
                            _ => panic!("Неожиданный формат для uvs"),
                        };

                        let indices = match chunk_data.mesh.indices().unwrap() {
                            bevy::render::mesh::Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
                            bevy::render::mesh::Indices::U32(indices) => indices.clone(),
                        };

                        crate::core::map::generator::mesh_generator::TerrainMeshData {
                            positions,
                            normals,
                            uvs,
                            indices,
                        }
                    };

                    let mesh_handle = mesh_pool.update_and_cache_mesh(
                        entity,
                        &chunk.id,
                        lod_level,
                        &mesh_data,
                        &mut meshes
                    );

                    mesh3d.0 = mesh_handle;

                    // Обновляем состояние чанка - ВАЖНО!
                    chunk.current_lod = Some(lod_level);

                    let elapsed_time = timer.elapsed();
                    #[cfg(debug_assertions)]
                    println!("Чанк {} загружен с LOD {:?} за {:?}",
                             chunk.id, lod_level, elapsed_time);
                }

                processed_tasks += 1;
                if processed_tasks >= max_tasks_per_frame {
                    return;
                }
            },
            BackgroundTaskResult::ChunkGenerated(chunk_data) => {
                if let Ok((_, _, mut chunk)) = q.get_mut(chunk_data.entity) {
                    chunk.generated = true;
                    #[cfg(debug_assertions)]
                    println!("Чанк {} сгенерирован", chunk.id);
                }
            }
        }
    }
}