use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use crate::core::map::generator::cache::terrain_mesh_cache;
use crate::core::map::generator::mesh_generator::TerrainMeshData;

pub fn load_terrain_mesh(chunk_id: &str) -> Mesh {
    let mesh_data = load_from_file(terrain_mesh_cache(chunk_id)).unwrap();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);
    mesh.insert_indices(Indices::U32(mesh_data.indices));

    mesh
}

fn load_from_file(path: PathBuf) -> std::io::Result<TerrainMeshData> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mesh: TerrainMeshData = bincode::deserialize(&buffer).unwrap();
    Ok(mesh)
}
