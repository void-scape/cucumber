use super::{collision::AbsoluteCollider, collision::Collider, collision::StaticBody};
use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Debug, Clone, Copy)]
pub struct SpatialData<D> {
    pub entity: Entity,
    pub collider: AbsoluteCollider,
    pub data: D,
}

impl<D> SpatialData<D> {
    pub fn from_entity(
        entity: Entity,
        transform: &GlobalTransform,
        collider: &Collider,
        data: D,
    ) -> Self {
        Self {
            collider: collider.global_absolute(transform),
            entity,
            data,
        }
    }
}

#[derive(Debug, Component)]
pub struct SpatialHash<D = ()> {
    cell_size: f32,
    objects: HashMap<(i32, i32), Vec<SpatialData<D>>>,
}

#[allow(dead_code)]
impl<D: Clone> SpatialHash<D> {
    pub fn new(cell_size: f32) -> Self {
        SpatialHash {
            cell_size,
            objects: HashMap::default(),
        }
    }

    pub fn new_with(cell_size: f32, data: impl IntoIterator<Item = SpatialData<D>>) -> Self {
        let mut slf = Self::new(cell_size);
        data.into_iter().for_each(|d| slf.insert(d));

        slf
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    fn hash(&self, position: &Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    /// Inserts spatial data into map.
    ///
    /// Data will be added to all cells overlapped by data's [`AbsoluteCollider`].
    pub fn insert(&mut self, data: SpatialData<D>) {
        let (min_x, min_y) = self.hash(&Vec2::new(data.collider.min_x(), data.collider.min_y()));
        let (max_x, max_y) = self.hash(&Vec2::new(data.collider.max_x(), data.collider.max_y()));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.objects.entry((x, y)).or_default().push(data.clone());
            }
        }
    }

    pub fn remove(&mut self, collider: AbsoluteCollider) {
        let (min_x, min_y) = self.hash(&Vec2::new(collider.min_x(), collider.min_y()));
        let (max_x, max_y) = self.hash(&Vec2::new(collider.max_x(), collider.max_y()));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let _ = self.objects.remove(&(x, y));
            }
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn objects_in_cell_mut<'a>(
        &'a mut self,
        position: &Vec2,
    ) -> impl Iterator<Item = &'a mut SpatialData<D>> + 'a {
        let cell = self.hash(position);

        self.objects
            .get_mut(&(cell.0, cell.1))
            .into_iter()
            .flatten()
    }

    pub fn nearby_objects<'a>(
        &'a self,
        position: &Vec2,
    ) -> impl Iterator<Item = &'a SpatialData<D>> + 'a {
        let cell = self.hash(position);

        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dy| {
                self.objects
                    .get(&(cell.0 + dx, cell.1 + dy))
                    .into_iter()
                    .flatten()
            })
        })
    }

    /// Returns false if encountered collision along path.
    pub fn ray_cast(&self, start: Vec2, end: Vec2, samples: usize) -> bool {
        let samples = (0..samples)
            .map(|i| start.lerp(end, i as f32 / samples as f32))
            .collect::<Vec<_>>();
        for sample in samples.iter() {
            if let Some(cells) = self.objects.get(&(self.hash(sample))) {
                for cell in cells.iter() {
                    for sample in samples.iter() {
                        if cell.collider.contains(sample) {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn nearby_objects_mut<'a>(
        &'a mut self,
        position: &Vec2,
    ) -> impl Iterator<Item = &'a mut SpatialData<D>> + 'a {
        let cell = self.hash(position);

        let dx = cell.0 - 1..=cell.0 + 1;
        let dy = cell.1 - 1..=cell.1 + 1;

        self.objects
            .iter_mut()
            .filter_map(move |(k, v)| (dx.contains(&k.0) && dy.contains(&k.1)).then_some(v))
            .flatten()
    }
}

pub fn store_static_body_in_spatial_map(
    mut hash: Query<(&mut SpatialHash, &Children)>,
    static_body: Query<(Entity, &GlobalTransform, &Collider), Added<StaticBody>>,
) {
    for (mut map, children) in hash.iter_mut() {
        for child in children.iter() {
            if let Ok((entity, global_transform, collider)) = static_body.get(*child) {
                map.insert(SpatialData {
                    collider: collider.global_absolute(global_transform),
                    data: (),
                    entity,
                })
            }
        }
    }
}
