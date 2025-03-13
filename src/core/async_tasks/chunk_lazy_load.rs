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

                if let Ok((entity, mut mesh3d, chunk)) = q.get_mut(chunk_data.entity) {
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
                        &mesh_data,
                        &mut meshes
                    );

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
