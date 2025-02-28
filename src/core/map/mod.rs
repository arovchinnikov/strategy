mod camera;

use std::cmp::min;
use std::f32::consts::PI;
use std::path::Path;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::palettes::basic::WHITE;
use bevy::math::vec3;
use bevy::pbr::wireframe::{Wireframe};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{default, Color, Commands, DirectionalLight, Mesh, Mesh3d, MeshMaterial3d, Quat, ResMut, Transform, Vec3};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use image::{GrayImage, ImageReader};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        camera::build(app);
        
        app.add_systems(Startup, init);
    }
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let width = 8192.0;
    let height = 3072.0;
    let chunk_width = 512.0;
    let chunk_height = 512.0;

    let num_chunks_x = (width / chunk_width) as u32;
    let num_chunks_z = (height / chunk_height) as u32;

    let subdivisions_x = 256;
    let subdivisions_z = 256;

    let heightmap = load_image_sync("common/map/heightmap.png").into_luma8();

    for z in 0..num_chunks_z {
        for x in 0..num_chunks_x {
            let start_x = x * chunk_width as u32;
            let start_z = z * chunk_height as u32;

            let plane_mesh = generate_terrain_mesh(
                start_x as f32,
                start_z as f32,
                chunk_width,
                chunk_height,
                subdivisions_x,
                subdivisions_z,
                &heightmap,
            );

            commands.spawn((
                Mesh3d::from(meshes.add(plane_mesh)),
                MeshMaterial3d::from(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.5, 0.4),
                    ..default()
                })),
                Transform {
                    translation: Vec3::new(start_x as f32 * 0.5, 0.0, start_z as f32 * 0.5),
                    scale: Vec3::new(0.5, 0.5, 0.5),
                    ..default()
                },
                Wireframe
            ));
        }
    }

    commands.spawn((
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2000.0, 0.0)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
    ));
}

fn generate_terrain_mesh(
    start_x: f32,
    start_z: f32,
    width: f32,
    height: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
    heightmap: &GrayImage,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let vert_count = ((subdivisions_x + 1) * (subdivisions_z + 1)) as usize;
    let mut positions = Vec::with_capacity(vert_count);
    let mut normals = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    // Определяем шаг по каждой оси и начальную позицию (центрированная плоскость)
    let step_x = width / subdivisions_x as f32;
    let step_y = height / subdivisions_z as f32;

    // Генерируем вершины
    for y in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let pos_x = x as f32 * step_x;
            let pos_z = y as f32 * step_y;

            let pixel_x = min((start_x + pos_x) as u32, heightmap.width() - 1);
            let pixel_z = min((start_z + pos_z) as u32, heightmap.height() - 1);

            let pos_y = heightmap.get_pixel(pixel_x, pixel_z)[0] as f32 * 0.35;
            
            positions.push([pos_x, pos_y, pos_z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([x as f32 / subdivisions_x as f32, y as f32 / subdivisions_z as f32]);
        }
    }

    // Генерируем индексы для треугольников
    let mut indices = Vec::new();
    for y in 0..subdivisions_z {
        for x in 0..subdivisions_x {
            let i = y * (subdivisions_x + 1) + x;
            // Первый треугольник
            indices.push(i);
            indices.push(i + subdivisions_x + 1);
            indices.push(i + 1);
            // Второй треугольник
            indices.push(i + 1);
            indices.push(i + subdivisions_x + 1);
            indices.push(i + subdivisions_x + 2);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}


fn load_image_sync(path: &str) -> image::DynamicImage {
    let img = ImageReader::open(Path::new(path))
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image");

    img
}
