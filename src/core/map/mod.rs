mod camera;
mod sea;
mod light;
mod generator;

use std::fs::File;
use std::io::{Read, Write};
use crate::core::map::generator::{generate_terrain_mesh, TerrainMeshData};
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetId, Assets, RenderAssetUsages};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{default, BuildChildren, Color, Commands, Component, GlobalTransform, Mesh, Mesh3d, MeshMaterial3d, Parent, ResMut, Transform, Vec3, Visibility};
use image::{GrayImage, ImageReader};
use std::path::Path;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::view::RenderLayers;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        camera::build(app);
        sea::build(app);
        light::build(app);
        
        app.add_systems(Startup, init);
    }
}


#[derive(Component)]
struct WorldMap {
    chunks_with: u32,
    chunks_height: u32,
    chunk_size: u32,
}

#[derive(Component)]
struct WorldChunk {
    id: u32,
    mesh_id: AssetId<Mesh>,
    loaded: bool
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    let mut chunk_id = 0;
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

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, terrain_mesh.positions.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, terrain_mesh.normals.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, terrain_mesh.uvs.clone());
            mesh.insert_indices(Indices::U32(terrain_mesh.indices.clone()));

            let filepath = format!("./tmp/cache/chunk_{}.mesh", chunk_id);

            save_to_bin(&terrain_mesh, filepath.as_str()).unwrap();

            let mesh_asset = meshes.add(mesh);
            let mesh_id = mesh_asset.id().clone();

            let terrain_chunk = commands.spawn((
                Mesh3d::from(mesh_asset),
                WorldChunk {
                    id: chunk_id,
                    mesh_id: mesh_id,
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

            meshes.remove(mesh_id);

            commands.entity(parent_entity).insert_children(chunk_id as usize, &[terrain_chunk]);

            chunk_id += 1;
        }
    }
}

fn save_to_bin(mesh: &TerrainMeshData, path: &str) -> std::io::Result<()> {
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
