use bevy::prelude::Component;

#[derive(Component)]
pub struct WorldMap {
    pub(crate) chunks_with: u32,
    pub(crate) chunks_height: u32,
    pub(crate) chunk_size: u32,
}

#[derive(Component)]
pub struct WorldChunk {
    pub(crate) id: String,
    pub(crate) loaded: bool
}