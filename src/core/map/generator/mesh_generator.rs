use bevy::math::Vec3;
use bincode::{Decode, Encode};
use image::GrayImage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone)]
pub struct TerrainMeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

use crate::core::map::generator::cache::LodLevel;

pub fn generate_terrain_mesh(
    start_x: f32,
    start_z: f32,
    width: f32,
    height: f32,
    lod_level: LodLevel,
    heightmap: &GrayImage,
) -> TerrainMeshData {
    let base_subdivisions = 256;

    let subdivisions_factor = match lod_level {
        LodLevel::High => 1,
        LodLevel::Medium => 2,
        LodLevel::Low => 4,
    };

    let subdivisions_x = base_subdivisions / subdivisions_factor;
    let subdivisions_z = base_subdivisions / subdivisions_factor;

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

    let flat_threshold = 0.0;

    let mut indices = Vec::with_capacity((subdivisions_x * subdivisions_z * 6) as usize);

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

    for i in (0..indices.len()).step_by(3) {
        let i1 = indices[i] as usize;
        let i2 = indices[i+1] as usize;
        let i3 = indices[i+2] as usize;

        let p1 = Vec3::new(positions[i1][0], positions[i1][1], positions[i1][2]);
        let p2 = Vec3::new(positions[i2][0], positions[i2][1], positions[i2][2]);
        let p3 = Vec3::new(positions[i3][0], positions[i3][1], positions[i3][2]);

        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2).normalize();

        normals[i1][0] += normal.x;
        normals[i1][1] += normal.y;
        normals[i1][2] += normal.z;

        normals[i2][0] += normal.x;
        normals[i2][1] += normal.y;
        normals[i2][2] += normal.z;

        normals[i3][0] += normal.x;
        normals[i3][1] += normal.y;
        normals[i3][2] += normal.z;
    }

    for normal in &mut normals {
        let length = (normal[0]*normal[0] + normal[1]*normal[1] + normal[2]*normal[2]).sqrt();
        if length > 0.001 {
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        }
    }

    let vertices_per_row = subdivisions_x + 1;
    let vertices_per_column = subdivisions_z + 1;

    let mut problem_vertices = Vec::new();
    for i in 0..positions.len() {
        let normal_vec = Vec3::new(normals[i][0], normals[i][1], normals[i][2]);
        if normal_vec.length() < 0.001 {
            problem_vertices.push(i);
        }
    }

    for vertex_idx in problem_vertices {
        let mut valid_neighbors = Vec::new();
        let row = vertex_idx / (vertices_per_row as usize);
        let col = vertex_idx % (vertices_per_row as usize);

        if row > 0 {
            let upper_idx = (row - 1) * (vertices_per_row as usize) + col;
            if Vec3::new(normals[upper_idx][0], normals[upper_idx][1], normals[upper_idx][2]).length() > 0.001 {
                valid_neighbors.push(upper_idx);
            }
        }

        if row < (vertices_per_column as usize) - 1 {
            let lower_idx = (row + 1) * (vertices_per_row as usize) + col;
            if Vec3::new(normals[lower_idx][0], normals[lower_idx][1], normals[lower_idx][2]).length() > 0.001 {
                valid_neighbors.push(lower_idx);
            }
        }

        if col > 0 {
            let left_idx = row * (vertices_per_row as usize) + (col - 1);
            if Vec3::new(normals[left_idx][0], normals[left_idx][1], normals[left_idx][2]).length() > 0.001 {
                valid_neighbors.push(left_idx);
            }
        }

        if col < (vertices_per_row as usize) - 1 {
            let right_idx = row * (vertices_per_row as usize) + (col + 1);
            if Vec3::new(normals[right_idx][0], normals[right_idx][1], normals[right_idx][2]).length() > 0.001 {
                valid_neighbors.push(right_idx);
            }
        }

        if !valid_neighbors.is_empty() {
            let mut avg_normal = Vec3::ZERO;
            for neighbor_idx in valid_neighbors {
                avg_normal += Vec3::new(
                    normals[neighbor_idx][0],
                    normals[neighbor_idx][1],
                    normals[neighbor_idx][2]
                );
            }

            if avg_normal.length() > 0.001 {
                avg_normal = avg_normal.normalize();
                normals[vertex_idx][0] = avg_normal.x;
                normals[vertex_idx][1] = avg_normal.y;
                normals[vertex_idx][2] = avg_normal.z;
            } else {
                normals[vertex_idx] = [0.0, 1.0, 0.0];
            }
        } else {
            normals[vertex_idx] = [0.0, 1.0, 0.0];
        }
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
