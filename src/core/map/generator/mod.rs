use crate::core::map::components::{WorldChunk, WorldMap};
use crate::core::map::generator::cache::{terrain_mesh_cache, terrain_mesh_cache_dir};
use crate::core::map::generator::mesh_generator::{generate_terrain_mesh, TerrainMeshData};
use crate::pkg::str::generate_short_hash;
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{default, App, BuildChildren, Commands, Component, GlobalTransform, Mesh3d, ResMut, Startup, Transform, Visibility};
use bevy::render::view::RenderLayers;
use image::{GrayImage, ImageReader};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::pkg::dir::init_dir;

mod mesh_generator;
pub(crate) mod mesh_loader;
mod cache;

pub fn build(app: &mut App) {
    app.add_systems(Startup, setup);
}

fn setup() {
    init_dir(terrain_mesh_cache_dir()).expect("error to init terrain mesh cache directory");
}

pub fn generate_terrain(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let width = 8192;
    let height = 4096;
    let chunk_size = 256;
    let pixels_per_vertex = 1;

    let num_chunks_x = width / chunk_size;
    let num_chunks_z = height / chunk_size;

    let heightmap = load_heightmap("common/map/heightmap.png");

    let parent_entity = commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        WorldMap {
            chunk_size: chunk_size,
            chunks_with: num_chunks_x,
            chunks_height: num_chunks_z
        }
    )).id();

    let mut chunk_num_id = 0;
    let subdivisions = chunk_size / pixels_per_vertex;

    for z in 0..num_chunks_z {
        for x in 0..num_chunks_x {
            let start_x = x * chunk_size;
            let start_z = z * chunk_size;

            let terrain_mesh = &generate_terrain_mesh(
                start_x as f32,
                start_z as f32,
                chunk_size as f32,
                chunk_size as f32,
                subdivisions,
                subdivisions,
                &heightmap,
            );

            let chunk_id = generate_short_hash(&chunk_num_id.to_string());

            let path = terrain_mesh_cache(chunk_id.as_str());

            save_to_bin(&terrain_mesh, path).unwrap();

            let terrain_chunk = commands.spawn((
                Mesh3d::from(Handle::default()),
                WorldChunk {
                    id: chunk_id,
                    loaded: false,
                },
                MeshMaterial3d::from(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.4),
                    perceptual_roughness: 1.0,
                    ..default()
                })),
                Transform {
                    translation: Vec3::new(start_x as f32, 0.0, start_z as f32),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                    ..default()
                },
                RenderLayers::layer(1)
            )).id();

            commands.entity(parent_entity).insert_children(chunk_num_id as usize, &[terrain_chunk]);

            chunk_num_id += 1;
        }
    }
}

fn save_to_bin(mesh: &TerrainMeshData, path: PathBuf) -> std::io::Result<()> {
    let encoded = bincode::serialize(mesh).unwrap();
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