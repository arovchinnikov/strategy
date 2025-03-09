use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use bevy::asset::RenderAssetUsages;
use crate::core::map::camera::CameraCorners;
use crate::core::map::{WorldChunk, WorldMap};
use bevy::prelude::{Assets, Commands, Entity, Mesh, Mesh3d, Query, Res, Transform, With};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::view::RenderLayers;
use serde::{Deserialize, Serialize};
use crate::core::map::generator::TerrainMeshData;

pub fn view_world(
    camera_corners: Query<&CameraCorners>,
    map: Query<&WorldMap>,
    mut chunks: Query<(Entity, &Transform, &mut RenderLayers, &mut Mesh3d, &mut WorldChunk)>,
    mut commands: Commands
) {
    let corners =  camera_corners.single();
    let map =  map.single();

    for (entity, transform, mut render_layers, mesh, mut chunk) in chunks.iter_mut() {
        let pos_x = transform.translation.x;
        let pos_z = transform.translation.z;

        let additional_space = 64.0;

        let mut loaded = false;

        let visible = in_view(corners, pos_x, pos_z, map.chunk_size as f32, additional_space);

        if visible {
            *render_layers = RenderLayers::from_layers(&[0, 1]);
            loaded = true;
        } else {
            *render_layers = RenderLayers::layer(1);
        }

        if loaded == false {
            loaded = in_view(corners, pos_x, pos_z, map.chunk_size as f32, additional_space + 2.0 * map.chunk_size as f32);
        }
        
        let filename = format!("./tmp/cache/chunk_{}.mesh", chunk.id);

        if chunk.loaded && !loaded {
        }

        if !chunk.loaded && loaded {}
    }
}

fn load_from_bin(path: &str) -> std::io::Result<TerrainMeshData> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mesh: TerrainMeshData = bincode::deserialize(&buffer).unwrap();
    Ok(mesh)
}

fn from_terrain_mesh_data(mesh_data: TerrainMeshData) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);
    mesh.insert_indices(Indices::U32(mesh_data.indices));
    mesh
}

fn in_view(
    corners: &CameraCorners,
    pos_x: f32,
    pos_z: f32,
    chunk_size: f32,
    additional_space: f32
) -> bool {
    let should_be_visible = !(pos_x + chunk_size + additional_space <= corners.min_x
        || pos_x - additional_space >= corners.max_x
        || pos_z + chunk_size + additional_space <= corners.min_z
        || pos_z - additional_space >= corners.max_z);

    should_be_visible
}
