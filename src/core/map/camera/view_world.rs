use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem, ChunkData};
use crate::core::map::camera::CameraCorners;
use crate::core::map::generator::TerrainMeshData;
use crate::core::map::{WorldChunk, WorldMap};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::{AssetId, Assets, Entity, Handle, Mesh, Mesh2d, Mesh3d, Query, ResMut, Resource, Transform};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::view::RenderLayers;
use bevy::tasks::AsyncComputeTaskPool;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

#[derive(Default, Resource)]
pub struct PendingMeshDeletions(Vec<Entity>);

pub fn process_pending_mesh_deletions(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Mesh3d)>,
    mut pending_deletions: ResMut<PendingMeshDeletions>,
) {
    if let Some(entity) = pending_deletions.0.pop() {
        let timer = Instant::now();
        if let Ok((_, mut mesh3d)) = query.get_mut(entity) {
            let mesh_id = mesh3d.0.id();
            
            mesh3d.0 = Handle::default();
            

            if meshes.contains(mesh_id) {
                meshes.remove_untracked(mesh_id);
            }
        }

        let remove_time = timer.elapsed();
        println!("chunk unloaded {:?}", remove_time);
    }
}

pub fn view_world(
    camera_corners: Query<&CameraCorners>,
    map: Query<&WorldMap>,
    mut chunks: Query<(Entity, &Transform, &mut RenderLayers, &mut WorldChunk, &mut Mesh3d)>,
    task_system: ResMut<BackgroundTaskSystem>,
    mut pending_deletions: ResMut<PendingMeshDeletions>
) {
    let corners =  camera_corners.single();
    let map =  map.single();

    for (entity, transform, mut render_layers, mut chunk, mut mesh) in chunks.iter_mut() {
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
            loaded = in_view(corners, pos_x, pos_z, map.chunk_size as f32, additional_space + map.chunk_size as f32);
        }
        
        let filepath = format!("./tmp/cache/chunk_{}.mesh", chunk.id);

        if chunk.loaded && !loaded {
            chunk.loaded = false;
            pending_deletions.0.push(entity);
            continue;
        }

        if !chunk.loaded && loaded {
            chunk.loaded = true;

            let sender = task_system.sender.clone();

            AsyncComputeTaskPool::get().spawn(async move {
                let mesh_data = load_from_bin(filepath.as_str()).unwrap();

                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);
                mesh.insert_indices(Indices::U32(mesh_data.indices));

                let chunk_data = ChunkData {
                    entity,
                    mesh
                };

                sender.send(BackgroundTaskResult::ChunkLoaded(chunk_data)).unwrap();
            }).detach();
        }
    }
}

fn load_from_bin(path: &str) -> std::io::Result<TerrainMeshData> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mesh: TerrainMeshData = bincode::deserialize(&buffer).unwrap();
    Ok(mesh)
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
