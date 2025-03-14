use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use crate::core::async_tasks::ChunkData;
use crate::core::map::components::WorldChunk;
use crate::core::map::terrain::mesh_generator::TerrainMeshData;
use crate::core::map::terrain::cache::LodLevel;
use crate::core::map::terrain::mesh_pool::MeshPool;

pub fn process_loaded_chunk(
    chunk_data: ChunkData,
    meshes: &mut ResMut<Assets<Mesh>>,
    mesh_pool: &mut ResMut<MeshPool>,
    q: &mut Query<(Entity, &mut Mesh3d, &mut WorldChunk)>,
) -> bool {
    let lod_level = chunk_data.lod.unwrap_or_else(|| {
        LodLevel::High
    });

    if let Ok((
        entity, 
        mut mesh3d, 
        mut chunk
    )) = q.get_mut(chunk_data.entity) {
        if !chunk.loaded || chunk.target_lod != Some(lod_level) {
            return false;
        }

        let mesh_data = extract_mesh_data_from_chunk(&chunk_data.mesh);
        let mesh_handle = mesh_pool.update_and_cache_mesh(
            entity,
            &chunk.id,
            lod_level,
            &mesh_data,
            meshes
        );

        mesh3d.0 = mesh_handle;
        chunk.current_lod = Some(lod_level);
        return true;
    }

    false
}

fn extract_mesh_data_from_chunk(mesh: &Mesh) -> TerrainMeshData {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(values) => values.clone(),
        _ => panic!("Invalid positions format"),
    };

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() {
        VertexAttributeValues::Float32x3(values) => values.clone(),
        _ => panic!("Invalid normals format"),
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() {
        VertexAttributeValues::Float32x2(values) => values.clone(),
        _ => panic!("Invalid uvs format"),
    };

    let indices = match mesh.indices().unwrap() {
        Indices::U16(indices) => indices.iter().map(|&i| i as u32).collect(),
        Indices::U32(indices) => indices.clone(),
    };

    TerrainMeshData {
        positions,
        normals,
        uvs,
        indices,
    }
}
