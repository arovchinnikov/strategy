use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem, ChunkData};
use crate::core::map::camera::{determine_lod_level, CameraCorners, CameraLodState};
use crate::core::map::components::{WorldChunk, WorldMap};
use crate::core::map::generator::cache::LodLevel;
use crate::core::map::generator::mesh_loader::load_terrain_mesh;
use crate::core::map::generator::mesh_pool::MeshPool;
use bevy::prelude::{Assets, Entity, Handle, Mesh, Mesh3d, Query, Res, ResMut, Resource, Transform};
use bevy::render::view::RenderLayers;
use bevy::tasks::AsyncComputeTaskPool;

#[derive(Default, Resource)]
pub struct PendingMeshDeletions(Vec<Entity>);

#[derive(Clone)]
pub struct PendingLodChange {
    entity: Entity,
    chunk_id: String,
    lod_level: LodLevel,
}

#[derive(Default, Resource)]
pub struct PendingLodChanges(Vec<PendingLodChange>);

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
            mesh_pool.return_mesh(entity, &mut meshes);

            mesh3d.0 = Handle::default();

            deletions_counter = deletions_counter + 1;
        }
    }

    if deletions_counter > 0 {
        #[cfg(debug_assertions)]
        let (available, cached, active, max) = mesh_pool.stats();
        #[cfg(debug_assertions)]
        println!("Удалено {} чанков. Пул мешей: {}/{} свободно, {} в кэше, {} активно",
                 deletions_counter, available, max, cached, active);
    }
}

pub fn process_lod_changes(
    mut pending_lod_changes: ResMut<PendingLodChanges>,
    mut chunks: Query<&mut WorldChunk>,
    task_system: ResMut<BackgroundTaskSystem>,
) {
    let max_changes_per_frame = 5;

    if pending_lod_changes.0.is_empty() {
        return;
    }

    let mut unique_changes = Vec::new();
    let mut processed_entities = std::collections::HashSet::new();

    for change in pending_lod_changes.0.iter() {
        if processed_entities.contains(&change.entity) {
            continue;
        }

        if let Ok(chunk) = chunks.get(change.entity) {
            if chunk.loaded && chunk.target_lod == Some(change.lod_level) &&
                chunk.current_lod != Some(change.lod_level) {
                unique_changes.push(change.clone());
                processed_entities.insert(change.entity);
            }
        }
    }

    pending_lod_changes.0.clear();
    let change_count = unique_changes.len().min(max_changes_per_frame);

    #[cfg(debug_assertions)]
    if change_count > 0 {
        println!("Обработка {} уникальных изменений LOD из {}", change_count, unique_changes.len());
    }

    if change_count < unique_changes.len() {
        pending_lod_changes.0.extend(unique_changes.iter().skip(change_count).cloned());
    }

    for change in unique_changes.iter().take(change_count) {
        let sender = task_system.sender.clone();
        let chunk_id = change.chunk_id.clone();
        let lod = change.lod_level;
        let entity = change.entity;

        AsyncComputeTaskPool::get().spawn(async move {
            #[cfg(debug_assertions)]
            println!("Загрузка меша с LOD {:?} для чанка {}", lod, chunk_id);

            let loaded_mesh = load_terrain_mesh(chunk_id.as_str(), lod);

            let chunk_data = ChunkData {
                entity,
                mesh: loaded_mesh,
                lod: Some(lod),
            };

            if let Err(e) = sender.send(BackgroundTaskResult::ChunkLoaded(chunk_data)) {
                eprintln!("Ошибка отправки результата загрузки чанка: {:?}", e);
            } else {
                #[cfg(debug_assertions)]
                println!("Меш с LOD {:?} для чанка {} успешно отправлен в основной поток", lod, chunk_id);
            }
        }).detach();
    }
}

pub fn view_world(
    camera_corners: Query<&CameraCorners>,
    map: Query<&WorldMap>,
    mut chunks: Query<(Entity, &Transform, &mut RenderLayers, &mut WorldChunk)>,
    mut pending_deletions: ResMut<PendingMeshDeletions>,
    mut pending_lod_changes: ResMut<PendingLodChanges>,
    mut mesh_pool: ResMut<MeshPool>,
    mut query_mesh: Query<&mut Mesh3d>,
    lod_state: Res<CameraLodState>,
) {
    let corners = camera_corners.single();
    let map = map.single();

    let current_global_lod = determine_lod_level(lod_state.current_height, &lod_state.lod_thresholds);
    let view_distance_multiplier = match current_global_lod {
        LodLevel::High => 1.0,
        LodLevel::Medium => 2.0,
        LodLevel::Low => 4.0,
    };

    let base_additional_space = 64.0;
    let additional_space = base_additional_space * view_distance_multiplier;

    static mut LOG_COUNTER: usize = 0;

    unsafe {
        LOG_COUNTER = (LOG_COUNTER + 1) % 120;
        if LOG_COUNTER == 0 {
            #[cfg(debug_assertions)]
            println!("Камера: высота = {}, LOD = {:?}, доп. пространство = {}",
                     lod_state.current_height, current_global_lod, additional_space);
        }
    }

    for (entity, transform, mut render_layers, mut chunk) in chunks.iter_mut() {
        let pos_x = transform.translation.x;
        let pos_z = transform.translation.z;

        let in_view_area = in_view(corners, pos_x, pos_z, map.chunk_size as f32, additional_space);
        let in_extended_area = in_view(corners, pos_x, pos_z, map.chunk_size as f32,
                                       additional_space + map.chunk_size as f32 * view_distance_multiplier);

        let should_be_loaded = in_view_area || in_extended_area;
        let should_be_visible = in_view_area;

        if should_be_visible {
            *render_layers = RenderLayers::from_layers(&[0, 1]);
        } else {
            *render_layers = RenderLayers::layer(1);
        }

        if chunk.loaded && !should_be_loaded {
            chunk.loaded = false;
            chunk.current_lod = None;
            chunk.target_lod = None;
            pending_deletions.0.push(entity);

            #[cfg(debug_assertions)]
            println!("Чанк {} выгружен (вне зоны видимости)", chunk.id);
            continue;
        }

        if should_be_loaded && chunk.generated {
            let needs_loading = !chunk.loaded || chunk.current_lod != Some(current_global_lod);
            let already_pending = pending_lod_changes.0.iter().any(|change|
                change.entity == entity && change.lod_level == current_global_lod
            );

            let already_has_target_lod = chunk.target_lod == Some(current_global_lod);
            if needs_loading && !already_pending && !already_has_target_lod {
                if !chunk.loaded {
                    chunk.loaded = true;
                }

                chunk.target_lod = Some(current_global_lod);
                let cache_hit = if mesh_pool.has_cached_mesh(&chunk.id, current_global_lod) {
                    if let Some(cached_mesh_handle) = mesh_pool.get_cached_mesh(entity, &chunk.id, current_global_lod) {
                        if let Ok(mut mesh3d) = query_mesh.get_mut(entity) {
                            mesh3d.0 = cached_mesh_handle;
                            chunk.current_lod = Some(current_global_lod);

                            #[cfg(debug_assertions)]
                            println!("Использован кэшированный меш для чанка {} (LOD {:?})",
                                     chunk.id, current_global_lod);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !cache_hit {
                    pending_lod_changes.0.push(PendingLodChange {
                        entity,
                        chunk_id: chunk.id.clone(),
                        lod_level: current_global_lod,
                    });

                    #[cfg(debug_assertions)]
                    println!("Запланирована загрузка чанка {} с LOD {:?}",
                             chunk.id, current_global_lod);
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    if !pending_lod_changes.0.is_empty() && unsafe { LOG_COUNTER == 0 } {
        println!("Запланировано {} изменений LOD", pending_lod_changes.0.len());
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
