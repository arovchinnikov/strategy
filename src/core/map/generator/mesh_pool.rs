use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use std::collections::VecDeque;

/// Ресурс, управляющий пулом мешей для эффективного повторного использования памяти
#[derive(Resource)]
pub struct MeshPool {
    // Доступные меши в пуле
    available_meshes: VecDeque<Handle<Mesh>>,
    // Меши, которые сейчас используются (Entity -> Handle<Mesh>)
    active_meshes: HashMap<Entity, Handle<Mesh>>,
    // Настройки пула
    min_pool_size: usize,
    max_pool_size: usize,
}

impl Default for MeshPool {
    fn default() -> Self {
        Self {
            available_meshes: VecDeque::new(),
            active_meshes: HashMap::new(),
            min_pool_size: 10,  // Минимальное количество мешей в пуле
            max_pool_size: 50,  // Максимальное количество мешей в пуле
        }
    }
}

impl MeshPool {
    /// Создает новый пул мешей с указанными параметрами
    pub fn new(min_pool_size: usize, max_pool_size: usize) -> Self {
        Self {
            available_meshes: VecDeque::with_capacity(min_pool_size),
            active_meshes: HashMap::new(),
            min_pool_size,
            max_pool_size,
        }
    }

    /// Создает "пустой" меш с минимальным количеством данных для предотвращения паники
    fn create_dummy_mesh() -> Mesh {
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList,
                                 bevy::asset::RenderAssetUsages::default());

        // Создаем один треугольник как заглушку
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = vec![0, 1, 2];

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

        mesh
    }

    /// Инициализирует пул начальным количеством мешей
    pub fn initialize(&mut self, meshes: &mut Assets<Mesh>) {
        for _ in 0..self.min_pool_size {
            // Создаем меш-заглушку вместо пустого меша
            let mesh = meshes.add(Self::create_dummy_mesh());
            self.available_meshes.push_back(mesh);
        }

        println!("Пул мешей инициализирован с {} элементами", self.min_pool_size);
    }

    /// Получает меш из пула или создает новый
    pub fn get_mesh(&mut self, entity: Entity, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        if let Some(mesh) = self.available_meshes.pop_front() {
            self.active_meshes.insert(entity, mesh.clone());
            mesh
        } else {
            // Пул пуст, создаем новый меш-заглушку
            let mesh = meshes.add(Self::create_dummy_mesh());
            self.active_meshes.insert(entity, mesh.clone());
            mesh
        }
    }

    /// Возвращает меш в пул (не очищаем полностью)
    pub fn return_mesh(&mut self, entity: Entity, meshes: &mut Assets<Mesh>) {
        if let Some(mesh_handle) = self.active_meshes.remove(&entity) {
            // Проверяем, не превышает ли размер пула максимальное значение
            if self.available_meshes.len() < self.max_pool_size {
                // Заменяем меш на минимальную заглушку для экономии памяти
                if let Some(mesh) = meshes.get_mut(&mesh_handle) {
                    // Создаем заглушку с минимальным размером
                    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
                    let normals = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
                    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
                    let indices = vec![0, 1, 2];

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
                }

                // Возвращаем меш в пул
                self.available_meshes.push_back(mesh_handle);
            } else {
                // Если пул переполнен, удаляем меш
                meshes.remove(mesh_handle.id());
            }
        }
    }

    /// Обновляет существующий меш новыми данными
    pub fn update_mesh(&self, entity: Entity, mesh_data: &crate::core::map::generator::mesh_generator::TerrainMeshData, meshes: &mut Assets<Mesh>) -> bool {
        if let Some(mesh_handle) = self.active_meshes.get(&entity) {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                // Обновляем данные меша
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions.clone());
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals.clone());
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs.clone());
                mesh.insert_indices(bevy::render::mesh::Indices::U32(mesh_data.indices.clone()));
                return true;
            }
        }
        false
    }

    /// Возвращает статистику использования пула
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.available_meshes.len(), self.active_meshes.len(), self.max_pool_size)
    }
}

/// Системы для регистрации в приложении
pub fn build(app: &mut App) {
    app.init_resource::<MeshPool>()
        .add_systems(Startup, init_mesh_pool);
}

fn init_mesh_pool(
    mut mesh_pool: ResMut<MeshPool>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    mesh_pool.initialize(&mut meshes);
}