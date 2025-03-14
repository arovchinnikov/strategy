use crate::core::map::components::{WorldChunk, WorldMap};
use crate::core::map::terrain::cache::{terrain_mesh_cache, terrain_mesh_cache_dir, terrain_mesh_lod_dir, LodLevel};
use crate::core::map::terrain::mesh_generator::{generate_terrain_mesh, TerrainMeshData};
use crate::pkg::dir::init_dir;
use crate::pkg::str::generate_short_hash;
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{default, App, BuildChildren, Commands, Entity, GlobalTransform, Mesh3d, Res, ResMut, Startup, Transform, Visibility};
use bevy::render::view::RenderLayers;
use image::{GrayImage, ImageReader};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use crate::core::async_tasks::{BackgroundTaskResult, BackgroundTaskSystem, GeneratedChunkData};

pub(crate) mod mesh_generator;
pub(crate) mod mesh_loader;
pub(crate) mod mesh_pool;
pub(crate) mod cache;

pub fn build(app: &mut App) {
    app.add_systems(Startup, setup);
}

pub fn setup() {
    init_dir(terrain_mesh_cache_dir()).expect("ошибка при создании основной директории кэша террейна");

    for lod in LodLevel::all_levels() {
        init_dir(terrain_mesh_lod_dir(lod))
            .expect(&format!("ошибка при создании директории кэша для LOD {}", lod as usize));
    }
}

pub fn generate_terrain(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    task_system: Res<BackgroundTaskSystem>
) {
    let width = 8192;
    let height = 4096;
    let chunk_size = 256;

    let num_chunks_x = width / chunk_size;
    let num_chunks_z = height / chunk_size;

    let heightmap = load_heightmap("common/map/heightmap.png");

    let parent_entity = commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        WorldMap {
            chunk_size,
            chunks_with: num_chunks_x,
            chunks_height: num_chunks_z
        }
    )).id();

    let mut chunk_num_id = 0;
    let sender = task_system.sender.clone();

    for z in 0..num_chunks_z {
        for x in 0..num_chunks_x {
            let start_x = x * chunk_size;
            let start_z = z * chunk_size;

            let material_handle = materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.4),
                perceptual_roughness: 1.0,
                ..default()
            });

            let chunk_id = generate_short_hash(&chunk_num_id.to_string());

            let terrain_chunk = commands.spawn((
                Mesh3d::from(Handle::default()),
                WorldChunk {
                    id: chunk_id.clone(),
                    loaded: false,
                    generated: false,
                    current_lod: None,
                    target_lod: None,
                },
                MeshMaterial3d::from(material_handle),
                Transform {
                    translation: Vec3::new(start_x as f32, 0.0, start_z as f32),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                    ..default()
                },
                RenderLayers::layer(1)
            )).id();

            commands.entity(parent_entity).insert_children(chunk_num_id as usize, &[terrain_chunk]);

            spawn_multi_lod_mesh_generation(
                sender.clone(),
                terrain_chunk,
                chunk_id,
                heightmap.clone(),
                start_x as f32,
                start_z as f32,
                chunk_size as f32,
            );

            chunk_num_id += 1;
        }
    }
}

fn spawn_multi_lod_mesh_generation(
    sender: crossbeam_channel::Sender<BackgroundTaskResult>,
    entity: Entity,
    chunk_id: String,
    heightmap: GrayImage,
    start_x: f32,
    start_z: f32,
    chunk_size: f32,
) {
    thread::spawn(move || {
        for lod in LodLevel::all_levels() {
            let terrain_mesh = generate_terrain_mesh(
                start_x,
                start_z,
                chunk_size,
                chunk_size,
                lod,
                &heightmap,
            );

            let path = terrain_mesh_cache(chunk_id.as_str(), lod);
            if let Err(e) = save_to_bin(&terrain_mesh, path) {
                println!("Не удалось сохранить меш в кэш (LOD {}): {}", lod as usize, e);
            }
        }

        let result = BackgroundTaskResult::ChunkGenerated(GeneratedChunkData {entity});
        if let Err(e) = sender.send(result) {
            println!("Не удалось отправить результат генерации чанка в основной поток: {:?}", e);
        }
    });
}

fn save_to_bin(mesh: &TerrainMeshData, path: PathBuf) -> std::io::Result<()> {
    let config = bincode::config::standard();
    let encoded = bincode::encode_to_vec(mesh, config).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut file = File::create(path)?;
    file.write_all(&encoded)?;
    Ok(())
}

fn load_heightmap(path: &str) -> GrayImage {
    let img = ImageReader::open(Path::new(path))
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");

    img.into_luma8()
}