use bevy::asset::RenderAssetUsages;
use bevy::prelude::{default, Assets, Color, Commands, Mesh, Mesh3d, MeshMaterial3d, ResMut, StandardMaterial, Startup, Transform, Vec3};
use bevy::render::mesh::{Indices, PrimitiveTopology};

pub fn build(app: &mut bevy::prelude::App) {
    app.add_systems(Startup, init);
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sea_width = 8192.0;
    let sea_height = 4096.0;
    let sea_subdivisions_x = 32;
    let sea_subdivisions_z = 16;

    let sea_mesh = create_flat_mesh(sea_width, sea_height, sea_subdivisions_x, sea_subdivisions_z);

    commands.spawn((
        Mesh3d::from(meshes.add(sea_mesh)),
        MeshMaterial3d::from(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.3, 0.7),
            perceptual_roughness: 0.8,
            metallic: 0.0,
            ..default()
        })),
        Transform {
            translation: Vec3::new(4096.0, 7.0, 2048.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..default()
        },
    ));
}

fn create_flat_mesh(width: f32, height: f32, subdivisions_x: u32, subdivisions_z: u32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let vert_count = ((subdivisions_x + 1) * (subdivisions_z + 1)) as usize;
    let mut positions = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    let step_x = width / subdivisions_x as f32;
    let step_z = height / subdivisions_z as f32;

    for z in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let pos_x = x as f32 * step_x - width / 2.0;
            let pos_z = z as f32 * step_z - height / 2.0;
            positions.push([pos_x, 0.0, pos_z]);
            uvs.push([x as f32 / subdivisions_x as f32, z as f32 / subdivisions_z as f32]);
        }
    }

    let mut indices = Vec::new();
    for z in 0..subdivisions_z {
        for x in 0..subdivisions_x {
            let top_left = z * (subdivisions_x + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * (subdivisions_x + 1) + x;
            let bottom_right = bottom_left + 1;

            indices.extend(&[top_left, bottom_left, top_right, top_right, bottom_left, bottom_right]);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    let normals = vec![[0.0, 1.0, 0.0]; vert_count];
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}
