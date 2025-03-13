use std::path::PathBuf;
use crate::pkg::dir::cache_directory;

// Определение уровней LOD
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LodLevel {
    High = 0,   // Высокая детализация (LOD 0)
    Medium = 1, // Средняя детализация (LOD 1)
    Low = 2     // Низкая детализация (LOD 2)
}

impl LodLevel {
    pub fn directory_name(&self) -> &str {
        match self {
            LodLevel::High => "lod0",
            LodLevel::Medium => "lod1",
            LodLevel::Low => "lod2",
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(LodLevel::High),
            1 => Some(LodLevel::Medium),
            2 => Some(LodLevel::Low),
            _ => None,
        }
    }

    pub fn all_levels() -> Vec<LodLevel> {
        vec![LodLevel::High, LodLevel::Medium, LodLevel::Low]
    }
}

pub fn terrain_mesh_cache_dir() -> PathBuf {
    cache_directory().join("terrain")
}

pub fn terrain_mesh_lod_dir(lod: LodLevel) -> PathBuf {
    terrain_mesh_cache_dir().join(lod.directory_name())
}

pub fn terrain_mesh_cache(chunk_id: &str, lod: LodLevel) -> PathBuf {
    terrain_mesh_lod_dir(lod).join(format!("{}.mesh", chunk_id))
}
