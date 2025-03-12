use std::path::PathBuf;
use crate::pkg::dir::cache_directory;

pub fn terrain_mesh_cache_dir() -> PathBuf {
    cache_directory().join("terrain")
}

pub fn terrain_mesh_cache(chunk_id: &str) -> PathBuf {
    terrain_mesh_cache_dir().with_file_name(chunk_id.to_owned() + ".mesh")
}

