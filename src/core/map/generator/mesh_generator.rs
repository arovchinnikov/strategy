use bevy::math::Vec3;
use image::GrayImage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerrainMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub fn generate_terrain_mesh(
    start_x: f32,
    start_z: f32,
    width: f32,
    height: f32,
    subdivisions_x: u32,
    subdivisions_z: u32,
    heightmap: &GrayImage,
) -> TerrainMeshData {
    let vert_count = ((subdivisions_x + 1) * (subdivisions_z + 1)) as usize;
    let mut positions = Vec::with_capacity(vert_count);
    let mut uvs = Vec::with_capacity(vert_count);

    let step_x = width / subdivisions_x as f32;
    let step_z = height / subdivisions_z as f32;

    for y in 0..=subdivisions_z {
        for x in 0..=subdivisions_x {
            let pos_x = x as f32 * step_x;
            let pos_z = y as f32 * step_z;

            let pixel_x = ((start_x + pos_x) as u32).min(heightmap.width() - 1);
            let pixel_z = ((start_z + pos_z) as u32).min(heightmap.height() - 1);

            let pos_y = calc_height(heightmap.get_pixel(pixel_x, pixel_z)[0] as f32);

            positions.push([pos_x, pos_y, pos_z]);
            uvs.push([x as f32 / subdivisions_x as f32, y as f32 / subdivisions_z as f32]);
        }
    }

    let mut indices = Vec::new();
    let flat_threshold = 0.0;

    let initial_block_size = subdivisions_x.min(subdivisions_z);
    process_block(
        0,
        0,
        initial_block_size,
        subdivisions_x,
        subdivisions_z,
        width,
        height,
        start_x,
        start_z,
        heightmap,
        &mut indices,
        flat_threshold,
    );

    let mut normals = vec![[0.0; 3]; positions.len()];

    for i in 0..positions.len() {
        let local_x = positions[i][0];
        let local_z = positions[i][2];

        let global_x = start_x + local_x;
        let global_z = start_z + local_z;

        let left_x = (global_x - 1.0).max(0.0);
        let right_x = (global_x + 1.0).min(heightmap.width() as f32 - 1.0);
        let down_z = (global_z - 1.0).max(0.0);
        let up_z = (global_z + 1.0).min(heightmap.height() as f32 - 1.0);

        let h_left = get_height_global(left_x, global_z, heightmap);
        let h_right = get_height_global(right_x, global_z, heightmap);
        let delta_x = h_right - h_left;

        let h_down = get_height_global(global_x, down_z, heightmap);
        let h_up = get_height_global(global_x, up_z, heightmap);
        let delta_z = h_up - h_down;

        let normal = Vec3::new(-delta_x, 2.0, -delta_z).normalize();
        normals[i] = [normal.x, normal.y, normal.z];
    }

    TerrainMeshData {
        positions,
        normals,
        uvs,
        indices,
    }
}

fn process_block(
    x: u32,
    y: u32,
    block_size: u32,
    sub_x: u32,
    sub_z: u32,
    width: f32,
    height: f32,
    start_x: f32,
    start_z: f32,
    heightmap: &GrayImage,
    indices: &mut Vec<u32>,
    threshold: f32,
) {
    if x + block_size > sub_x || y + block_size > sub_z {
        return;
    }

    let mut h_min = f32::INFINITY;
    let mut h_max = f32::NEG_INFINITY;
    for j in y..=y + block_size {
        for i in x..=x + block_size {
            let h = get_height(i, j, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            h_min = h_min.min(h);
            h_max = h_max.max(h);
        }
    }
    let max_diff = h_max - h_min;

    if max_diff <= threshold {
        let has_steep_edge = check_edges(
            x, y, block_size,
            sub_x, sub_z,
            width, height,
            start_x, start_z,
            heightmap,
            threshold,
        );

        if !has_steep_edge {
            let top_left = y * (sub_x + 1) + x;
            let top_right = y * (sub_x + 1) + x + block_size;
            let bottom_left = (y + block_size) * (sub_x + 1) + x;
            let bottom_right = (y + block_size) * (sub_x + 1) + x + block_size;
            indices.extend(&[top_left, bottom_left, top_right, top_right, bottom_left, bottom_right]);
        } else {
            divide_block(x, y, block_size, sub_x, sub_z, width, height, start_x, start_z, heightmap, indices, threshold);
        }
    } else {
        divide_block(x, y, block_size, sub_x, sub_z, width, height, start_x, start_z, heightmap, indices, threshold);
    }
}

fn get_height(
    x: u32,
    y: u32,
    sub_x: u32,
    sub_z: u32,
    width: f32,
    height: f32,
    start_x: f32,
    start_z: f32,
    heightmap: &GrayImage,
) -> f32 {
    let step_x = width / sub_x as f32;
    let step_z = height / sub_z as f32;
    let pos_x = x as f32 * step_x;
    let pos_z = y as f32 * step_z;

    let pixel_x = ((start_x + pos_x) as u32).min(heightmap.width() - 1);
    let pixel_z = ((start_z + pos_z) as u32).min(heightmap.height() - 1);

    calc_height(heightmap.get_pixel(pixel_x, pixel_z)[0] as f32)
}

fn divide_block(
    x: u32,
    y: u32,
    block_size: u32,
    sub_x: u32,
    sub_z: u32,
    width: f32,
    height: f32,
    start_x: f32,
    start_z: f32,
    heightmap: &GrayImage,
    indices: &mut Vec<u32>,
    threshold: f32,
) {
    if block_size > 1 {
        let half = block_size / 2;
        process_block(x, y, half, sub_x, sub_z, width, height, start_x, start_z, heightmap, indices, threshold);
        process_block(x + half, y, half, sub_x, sub_z, width, height, start_x, start_z, heightmap, indices, threshold);
        process_block(x, y + half, half, sub_x, sub_z, width, height, start_x, start_z, heightmap, indices, threshold);
        process_block(x + half, y + half, half, sub_x, sub_z, width, height, start_x, start_z, heightmap, indices, threshold);
    } else {
        let i = y * (sub_x + 1) + x;
        indices.extend(&[i, i + sub_x + 1, i + 1, i + 1, i + sub_x + 1, i + sub_x + 2]);
    }
}

fn check_edges(
    block_x: u32,
    block_y: u32,
    block_size: u32,
    sub_x: u32,
    sub_z: u32,
    width: f32,
    height: f32,
    start_x: f32,
    start_z: f32,
    heightmap: &GrayImage,
    threshold: f32,
) -> bool {
    // Edges
    if block_x > 0 {
        for z in block_y..=block_y + block_size {
            let h_current = get_height(block_x, z, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            let h_neighbor = get_height(block_x - 1, z, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            if (h_current - h_neighbor).abs() > threshold {
                return true;
            }
        }
    }
    if block_x + block_size < sub_x {
        for z in block_y..=block_y + block_size {
            let h_current = get_height(block_x + block_size, z, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            let h_neighbor = get_height(block_x + block_size + 1, z, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            if (h_current - h_neighbor).abs() > threshold {
                return true;
            }
        }
    }
    if block_y > 0 {
        for x in block_x..=block_x + block_size {
            let h_current = get_height(x, block_y, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            let h_neighbor = get_height(x, block_y - 1, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            if (h_current - h_neighbor).abs() > threshold {
                return true;
            }
        }
    }
    if block_y + block_size < sub_z {
        for x in block_x..=block_x + block_size {
            let h_current = get_height(x, block_y + block_size, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            let h_neighbor = get_height(x, block_y + block_size + 1, sub_x, sub_z, width, height, start_x, start_z, heightmap);
            if (h_current - h_neighbor).abs() > threshold {
                return true;
            }
        }
    }
    // Corners
    if block_x > 0 && block_y > 0 {
        let h_current = get_height(block_x, block_y, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        let h_neighbor = get_height(block_x - 1, block_y - 1, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        if (h_current - h_neighbor).abs() > threshold {
            return true;
        }
    }
    if block_x + block_size < sub_x && block_y > 0 {
        let x = block_x + block_size;
        let h_current = get_height(x, block_y, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        let h_neighbor = get_height(x + 1, block_y - 1, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        if (h_current - h_neighbor).abs() > threshold {
            return true;
        }
    }
    if block_x > 0 && block_y + block_size < sub_z {
        let y = block_y + block_size;
        let h_current = get_height(block_x, y, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        let h_neighbor = get_height(block_x - 1, y + 1, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        if (h_current - h_neighbor).abs() > threshold {
            return true;
        }
    }
    if block_x + block_size < sub_x && block_y + block_size < sub_z {
        let x = block_x + block_size;
        let y = block_y + block_size;
        let h_current = get_height(x, y, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        let h_neighbor = get_height(x + 1, y + 1, sub_x, sub_z, width, height, start_x, start_z, heightmap);
        if (h_current - h_neighbor).abs() > threshold {
            return true;
        }
    }

    false
}

fn calc_height(height: f32) -> f32 {
    if height < 6.0 {
        return 0.0;
    }

    let sea_level_height = 16.0;

    if height <= sea_level_height {
        return height * 0.4
    }

    (sea_level_height * 0.4) + (height-sea_level_height) * 0.35
}

fn get_height_global(x: f32, z: f32, heightmap: &GrayImage) -> f32 {
    let px = x.min(heightmap.width() as f32 - 1.0).max(0.0) as u32;
    let pz = z.min(heightmap.height() as f32 - 1.0).max(0.0) as u32;
    calc_height(heightmap.get_pixel(px, pz)[0] as f32)
}
