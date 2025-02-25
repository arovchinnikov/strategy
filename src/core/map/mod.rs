use bevy::app::{App, Plugin, Startup};
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::pbr::wireframe::{Wireframe};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Camera3d, Color, Commands, Mesh, Mesh3d, MeshMaterial3d, ResMut, Transform, Vec3};
use bevy::render::mesh::{Indices, PrimitiveTopology};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
    }
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Камера
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5000.0, 1024.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Размеры плоскости
    let width = 8192.0;
    let height = 3616.0;
    let subdivisions_x = 256;

    // Сохраним пропорции плоскости:
    let subdivisions_y = ((subdivisions_x as f32) * (height / width)).round() as u32;

    let plane_mesh = generate_plane(width, height, subdivisions_x, subdivisions_y);

    commands.spawn((
        Mesh3d::from(meshes.add(plane_mesh)),
        MeshMaterial3d::from(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..Default::default()
        })),
        Transform {
            ..Default::default()
        },
        Wireframe
    ));
}

fn generate_plane(width: f32, height: f32, subdivisions_x: u32, subdivisions_y: u32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let vert_count = ((subdivisions_x + 1) * (subdivisions_y + 1)) as usize;
    let mut positions = Vec::with_capacity(vert_count);
    let mut normals = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    // Определяем шаг по каждой оси и начальную позицию (центрированная плоскость)
    let step_x = width / subdivisions_x as f32;
    let step_y = height / subdivisions_y as f32;
    let origin_x = -width / 2.0;
    let origin_y = -height / 2.0;

    // Генерируем вершины
    for y in 0..=subdivisions_y {
        for x in 0..=subdivisions_x {
            let pos_x = origin_x + x as f32 * step_x;
            let pos_y = 0.0;
            let pos_z = origin_y + y as f32 * step_y;
            positions.push([pos_x, pos_y, pos_z]);
            normals.push([0.0, 1.0, 0.0]); // нормаль вверх
            uvs.push([x as f32 / subdivisions_x as f32, y as f32 / subdivisions_y as f32]);
        }
    }

    // Генерируем индексы для треугольников
    let mut indices = Vec::new();
    for y in 0..subdivisions_y {
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