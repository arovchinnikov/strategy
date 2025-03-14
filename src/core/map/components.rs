use bevy::prelude::Component;
use crate::core::map::terrain::cache::LodLevel;

#[derive(Component)]
pub(crate) struct WorldMap {
    pub(crate) chunks_with: u32,
    pub(crate) chunks_height: u32,
    pub(crate) chunk_size: u32,
}

#[derive(Component)]
pub(crate) struct WorldChunk {
    pub id: String,
    pub loaded: bool,
    pub generated: bool,
    pub current_lod: Option<LodLevel>,
    pub target_lod: Option<LodLevel>,
}
