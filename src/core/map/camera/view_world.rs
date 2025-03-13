use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem, ChunkData};
use crate::core::map::camera::CameraCorners;
use crate::core::map::components::{WorldChunk, WorldMap};
use crate::core::map::generator::mesh_loader::load_terrain_mesh;
use bevy::prelude::{Assets, Entity, Handle, Mesh, Mesh3d, Query, ResMut, Resource, Transform};
use bevy::render::view::RenderLayers;
use bevy::tasks::AsyncComputeTaskPool;
use crate::core::map::generator::mesh_pool::MeshPool;

#[derive(Default, Resource)]
pub struct PendingMeshDeletions(Vec<Entity>);

pub fn process_pending_mesh_deletions(
    mut mesh_pool: ResMut<MeshPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Mesh3d)>,
    mut pending_deletions: ResMut<PendingMeshDeletions>,
) {
    let deletion_batch_size = 5;
    let delete_count = pending_deletions.0.len().min(deletion_batch_size);

    let entities_to_process: Vec<Entity> = pending_deletions.0.drain(0..delete_count).collect();

    let mut deletions_counter = 0;
    for entity in entities_to_process {
        if let Ok((entity, mut mesh3d)) = query.get_mut(entity) {
            // Возвращаем меш в пул вместо удаления
            mesh_pool.return_mesh(entity, &mut meshes);

            // Устанавливаем пустой хэндл
            mesh3d.0 = Handle::default();

            deletions_counter = deletions_counter + 1;
        }
    }

    if deletions_counter > 0 {
        #[cfg(debug_assertions)]
        let (available, active, max) = mesh_pool.stats();
        #[cfg(debug_assertions)]
        println!("Processed {} chunk unloads. Mesh pool: {}/{} available, {} active",
                 deletions_counter, available, max, active);
    }
}

pub fn view_world(
    camera_corners: Query<&CameraCorners>,
    map: Query<&WorldMap>,
    mut chunks: Query<(Entity, &Transform, &mut RenderLayers, &mut WorldChunk)>,
    task_system: ResMut<BackgroundTaskSystem>,
    mut pending_deletions: ResMut<PendingMeshDeletions>
) {
    let corners =  camera_corners.single();
    let map =  map.single();

    for (entity, transform, mut render_layers, mut chunk) in chunks.iter_mut() {
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

        if chunk.loaded && !loaded {
            chunk.loaded = false;
            pending_deletions.0.push(entity);
            continue;
        }

        if !chunk.loaded && loaded && chunk.generated {
            chunk.loaded = true;

            let sender = task_system.sender.clone();
            let chunk_id = chunk.id.clone();

            AsyncComputeTaskPool::get().spawn(async move {
                let loaded_mesh = load_terrain_mesh(chunk_id.as_str());

                let chunk_data = ChunkData {
                    entity,
                    mesh: loaded_mesh,
                };

                sender.send(BackgroundTaskResult::ChunkLoaded(chunk_data)).unwrap();
            }).detach();
        }
    }
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
