mod chunk_lazy_load;

use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::core::async_tasks::chunk_lazy_load::handle_background_tasks;

pub enum BackgroundTaskResult {
    ChunkLoaded(ChunkData)
}

pub struct ChunkData {
    pub entity: Entity,
    pub mesh: Mesh,
}

#[derive(Resource)]
pub struct BackgroundTaskSystem {
    pub sender: Sender<BackgroundTaskResult>,
    pub receiver: Receiver<BackgroundTaskResult>,
}

impl Default for BackgroundTaskSystem {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

pub fn build(app: &mut App) {
    app.init_resource::<BackgroundTaskSystem>();
    app.add_systems(Update, handle_background_tasks);
}
