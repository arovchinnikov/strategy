use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use std::collections::VecDeque;
use crate::core::map::terrain::cache::LodLevel;

#[derive(Resource)]
pub struct MeshPool {
    available_generic_meshes: VecDeque<Handle<Mesh>>,
    cached_chunk_meshes: HashMap<String, HashMap<LodLevel, Handle<Mesh>>>,
    active_meshes: HashMap<Entity, MeshUsageInfo>,
    min_pool_size: usize,
    max_pool_size: usize,
    max_cached_chunks: usize,
}

struct MeshUsageInfo {
    handle: Handle<Mesh>,
    chunk_id: Option<String>,
    lod_level: Option<LodLevel>,
}

impl Default for MeshPool {
    fn default() -> Self {
        Self {
            available_generic_meshes: VecDeque::new(),
            cached_chunk_meshes: HashMap::new(),
            active_meshes: HashMap::new(),
            min_pool_size: 10,
            max_pool_size: 50,
            max_cached_chunks: 100,
        }
    }
}

impl MeshPool {
    pub fn new(min_pool_size: usize, max_pool_size: usize, max_cached_chunks: usize) -> Self {
        Self {
            available_generic_meshes: VecDeque::with_capacity(min_pool_size),
            cached_chunk_meshes: HashMap::new(),
            active_meshes: HashMap::new(),
            min_pool_size,
            max_pool_size,
            max_cached_chunks,
        }
    }

    fn create_dummy_mesh() -> Mesh {
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());

        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = vec![0, 1, 2];

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        mesh
    }

    pub fn initialize(&mut self, meshes: &mut Assets<Mesh>) {
        for _ in 0..self.min_pool_size {
            let mesh = meshes.add(Self::create_dummy_mesh());
            self.available_generic_meshes.push_back(mesh);
        }

        println!("Пул мешей инициализирован с {} элементами", self.min_pool_size);
    }

    pub fn has_cached_mesh(&self, chunk_id: &str, lod: LodLevel) -> bool {
        if let Some(lod_map) = self.cached_chunk_meshes.get(chunk_id) {
            lod_map.contains_key(&lod)
        } else {
            false
        }
    }

    pub fn get_cached_mesh(&mut self, entity: Entity, chunk_id: &str, lod: LodLevel) -> Option<Handle<Mesh>> {
        if let Some(lod_map) = self.cached_chunk_meshes.get_mut(chunk_id) {
            if let Some(mesh_handle) = lod_map.remove(&lod) {
                self.active_meshes.insert(entity, MeshUsageInfo {
                    handle: mesh_handle.clone(),
                    chunk_id: Some(chunk_id.to_string()),
                    lod_level: Some(lod),
                });
                return Some(mesh_handle);
            }
        }
        None
    }

    pub fn get_mesh(&mut self, entity: Entity, chunk_id: Option<&str>, lod: Option<LodLevel>, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        if let (Some(id), Some(lod_level)) = (chunk_id, lod) {
            if let Some(mesh_handle) = self.get_cached_mesh(entity, id, lod_level) {
                return mesh_handle;
            }
        }

        if let Some(mesh) = self.available_generic_meshes.pop_front() {
            self.active_meshes.insert(entity, MeshUsageInfo {
                handle: mesh.clone(),
                chunk_id: chunk_id.map(|id| id.to_string()),
                lod_level: lod,
            });
            mesh
        } else {
            let mesh = meshes.add(Self::create_dummy_mesh());
            self.active_meshes.insert(entity, MeshUsageInfo {
                handle: mesh.clone(),
                chunk_id: chunk_id.map(|id| id.to_string()),
                lod_level: lod,
            });
            mesh
        }
    }

    pub fn return_mesh(&mut self, entity: Entity, meshes: &mut Assets<Mesh>) {
        if let Some(usage_info) = self.active_meshes.remove(&entity) {
            let mesh_handle = usage_info.handle;

            if let (Some(chunk_id), Some(lod)) = (usage_info.chunk_id, usage_info.lod_level) {
                if self.cached_chunk_meshes.len() < self.max_cached_chunks {
                    let lod_map = self.cached_chunk_meshes
                        .entry(chunk_id)
                        .or_insert_with(HashMap::new);

                    lod_map.insert(lod, mesh_handle);
                    return;
                }
            }

            if self.available_generic_meshes.len() < self.max_pool_size {
                if let Some(mesh) = meshes.get_mut(&mesh_handle) {
                    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
                    let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
                    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
                    let indices = vec![0, 1, 2];

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
                }

                self.available_generic_meshes.push_back(mesh_handle);
            } else {
                meshes.remove(mesh_handle.id());
            }
        }
    }

    pub fn update_and_cache_mesh(
        &mut self,
        entity: Entity,
        chunk_id: &str,
        lod: LodLevel,
        mesh_data: &crate::core::map::terrain::mesh_generator::TerrainMeshData,
        meshes: &mut Assets<Mesh>
    ) -> Handle<Mesh> {
        let mesh_handle = if let Some(usage_info) = self.active_meshes.get(&entity) {
            usage_info.handle.clone()
        } else {
            self.get_mesh(entity, Some(chunk_id), Some(lod), meshes)
        };

        if let Some(mesh) = meshes.get_mut(&mesh_handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs.clone());
            mesh.insert_indices(bevy::render::mesh::Indices::U32(mesh_data.indices.clone()));
        }

        self.active_meshes.insert(entity, MeshUsageInfo {
            handle: mesh_handle.clone(),
            chunk_id: Some(chunk_id.to_string()),
            lod_level: Some(lod),
        });

        let lod_map = self.cached_chunk_meshes
            .entry(chunk_id.to_string())
            .or_insert_with(HashMap::new);

        if !lod_map.contains_key(&lod) {
            let cached_mesh = meshes.add(Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                bevy::asset::RenderAssetUsages::default()
            ));

            if let Some(mesh) = meshes.get_mut(&cached_mesh) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions.clone());
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals.clone());
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs.clone());
                mesh.insert_indices(bevy::render::mesh::Indices::U32(mesh_data.indices.clone()));
            }

            lod_map.insert(lod, cached_mesh);

            #[cfg(debug_assertions)]
            println!("Меш чанка {} с LOD {:?} добавлен в кэш", chunk_id, lod);
        }

        mesh_handle
    }

    pub fn stats(&self) -> (usize, usize, usize, usize) {
        let total_cached = self.cached_chunk_meshes.values().map(|map| map.len()).sum::<usize>();

        (
            self.available_generic_meshes.len(),
            total_cached,
            self.active_meshes.len(),
            self.max_pool_size
        )
    }
}

pub fn build(app: &mut App) {
    app.init_resource::<MeshPool>()
        .add_systems(Startup, init_mesh_pool);
}

fn init_mesh_pool(
    mut mesh_pool: ResMut<MeshPool>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    mesh_pool.initialize(&mut meshes);
}
