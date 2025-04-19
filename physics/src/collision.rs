use super::{
    gravity::{BrushingLeft, BrushingRight, Grounded},
    layers,
    prelude::Velocity,
    spatial,
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
    utils::hashbrown::{HashMap, HashSet},
};
use spatial::SpatialHash;
use std::{cmp::Ordering, marker::PhantomData};

/// Contains a list of entities which a [`DynamicBody`] with [`layers::CollidesWith<T>`] collided
/// with this frame for the layer `T`.
#[derive(Component)]
pub struct Collision<T>(smallvec::SmallVec<[Entity; 4]>, PhantomData<T>);

impl<T> Default for Collision<T> {
    fn default() -> Self {
        Self(smallvec::SmallVec::default(), PhantomData)
    }
}

impl<T> Collision<T> {
    pub fn entities(&self) -> &[Entity] {
        &self.0
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

/// A vector describing the collision resolution applied to
/// this entity during collision checking, if any.
#[derive(Default, Component)]
pub struct TotalResolution(Vec2);

impl TotalResolution {
    pub fn get(&self) -> Vec2 {
        self.0
    }
}

pub(crate) fn clear_resolution(mut q: Query<&mut TotalResolution>) {
    for mut res in q.iter_mut() {
        res.0 = Vec2::new(0.0, 0.0);
    }
}

/// Marks this entity as having a static position throughout the lifetime of the program.
///
/// All [`StaticBody`] entities are added to a [`spatial::SpatialHash`] after spawning.
///
/// Moving a static body entity will NOT result in their collision being updated.
#[derive(Debug, Default, Clone, Copy, Component)]
#[require(Collider)]
#[component(on_remove = remove_static_body)]
pub struct StaticBody;

fn remove_static_body(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    if let Some(parent) = world.get::<Parent>(entity) {
        let Some(global_t) = world.get::<GlobalTransform>(entity) else {
            return;
        };
        let collider = world.get::<Collider>(entity).unwrap();
        let collider = collider.global_absolute(global_t);

        if let Some(mut hash) = world.get_mut::<SpatialHash>(parent.get()) {
            hash.remove(collider);
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
#[require(Collider)]
pub struct DynamicBody;

/// Prevents a dynamic body entity from being pushed.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Massive;

/// To check for collisions, first convert this enum into an [`AbsoluteCollider`]
/// with [`Collider::absolute`].
#[derive(Debug, Clone, Copy, PartialEq, Component)]
#[require(TotalResolution)]
pub enum Collider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl Default for Collider {
    fn default() -> Self {
        Self::from_rect(Vec2::ZERO, Vec2::ZERO)
    }
}

impl Collider {
    pub fn from_rect(tl: Vec2, size: Vec2) -> Self {
        Self::Rect(RectCollider { tl, size })
    }

    pub fn from_circle(position: Vec2, radius: f32) -> Self {
        Self::Circle(CircleCollider { position, radius })
    }

    pub fn absolute(&self, transform: &Transform) -> AbsoluteCollider {
        match self {
            Self::Rect(rect) => AbsoluteCollider::Rect(RectCollider {
                tl: rect.tl + transform.translation.xy(),
                size: rect.size,
            }),
            Self::Circle(circle) => AbsoluteCollider::Circle(CircleCollider {
                position: circle.position + transform.translation.xy(),
                radius: circle.radius,
            }),
        }
    }

    pub fn global_absolute(&self, transform: &GlobalTransform) -> AbsoluteCollider {
        match self {
            Self::Rect(rect) => AbsoluteCollider::Rect(RectCollider {
                tl: rect.tl + transform.translation().xy(),
                size: rect.size,
            }),
            Self::Circle(circle) => AbsoluteCollider::Circle(CircleCollider {
                position: circle.position + transform.translation().xy(),
                radius: circle.radius,
            }),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AbsoluteCollider {
    Rect(RectCollider),
    Circle(CircleCollider),
}

impl AbsoluteCollider {
    pub fn expand(&self, factor: f32) -> Self {
        match self {
            Self::Rect(rect) => Self::Rect(rect.expand(factor)),
            Self::Circle(circle) => Self::Circle(circle.expand(factor)),
        }
    }

    pub fn get_aabb(&self) -> RectCollider {
        match self {
            Self::Rect(rect) => *rect,
            Self::Circle(_) => todo!("implement aabb for circles"),
        }
    }

    pub fn position(&self) -> Vec2 {
        match self {
            Self::Rect(rect) => rect.tl,
            Self::Circle(circle) => circle.position,
        }
    }

    pub fn center(&self) -> Vec2 {
        match self {
            Self::Rect(r) => r.center(),
            Self::Circle(c) => c.position,
        }
    }

    pub fn contains(&self, point: &Vec2) -> bool {
        match self {
            Self::Rect(r) => r.contains(point),
            _ => todo!(),
        }
    }

    pub fn max_x(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.x + rect.size.x,
            Self::Circle(circle) => circle.position.x + circle.radius,
        }
    }

    pub fn min_x(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.x,
            Self::Circle(circle) => circle.position.x - circle.radius,
        }
    }

    pub fn max_y(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.y,
            Self::Circle(circle) => circle.position.y + circle.radius,
        }
    }

    pub fn min_y(&self) -> f32 {
        match self {
            Self::Rect(rect) => rect.tl.y - rect.size.y,
            Self::Circle(circle) => circle.position.y - circle.radius,
        }
    }
}

impl CollidesWith<Self> for AbsoluteCollider {
    fn collides_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rect(s), Self::Rect(o)) => s.collides_with(o),
            (Self::Rect(s), Self::Circle(o)) => s.collides_with(o),
            (Self::Circle(s), Self::Rect(o)) => s.collides_with(o),
            (Self::Circle(s), Self::Circle(o)) => s.collides_with(o),
        }
    }

    fn resolution(&self, other: &Self) -> Vec2 {
        match (self, other) {
            (Self::Rect(s), Self::Rect(o)) => s.resolution(o),
            (Self::Rect(s), Self::Circle(o)) => s.resolution(o),
            (Self::Circle(s), Self::Rect(o)) => s.resolution(o),
            (Self::Circle(s), Self::Circle(o)) => s.resolution(o),
        }
    }
}

pub trait CollidesWith<T> {
    fn collides_with(&self, other: &T) -> bool;
    fn resolution(&self, other: &T) -> Vec2;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
pub struct RectCollider {
    pub tl: Vec2,
    pub size: Vec2,
}

impl RectCollider {
    pub fn expand(mut self, factor: f32) -> Self {
        let center = self.center();
        self.size *= factor;
        let new_center = self.center();
        self.tl += center - new_center;
        self
    }

    pub fn contains(&self, point: &Vec2) -> bool {
        self.tl.x < point.x && self.br().x > point.x && self.tl.y > point.y && self.br().y < point.y
    }

    pub fn br(&self) -> Vec2 {
        Vec2::new(self.tl.x + self.size.x, self.tl.y - self.size.y)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.tl.x + self.size.x * 0.5, self.tl.y - self.size.y * 0.5)
    }

    pub fn line_intersection(&self, p0: Vec2, p1: Vec2) -> Option<Vec2> {
        // Parametric direction
        let dir = p1 - p0;

        // Boundaries
        let left = self.tl.x;
        let right = self.tl.x + self.size.x;
        let top = self.tl.y;
        let bottom = self.tl.y - self.size.y;

        let mut t_candidates = Vec::new();

        // 1) Left boundary (x = left), as long as dir.x != 0
        if dir.x != 0.0 {
            let t = (left - p0.x) / dir.x;
            if t >= 0.0 && t <= 1.0 {
                let y_at_t = p0.y + t * dir.y;
                if y_at_t <= top && y_at_t >= bottom {
                    t_candidates.push(t);
                }
            }
        }

        // 2) Right boundary (x = right)
        if dir.x != 0.0 {
            let t = (right - p0.x) / dir.x;
            if t >= 0.0 && t <= 1.0 {
                let y_at_t = p0.y + t * dir.y;
                if y_at_t <= top && y_at_t >= bottom {
                    t_candidates.push(t);
                }
            }
        }

        // 3) Top boundary (y = top)
        if dir.y != 0.0 {
            let t = (top - p0.y) / dir.y;
            if t >= 0.0 && t <= 1.0 {
                let x_at_t = p0.x + t * dir.x;
                if x_at_t >= left && x_at_t <= right {
                    t_candidates.push(t);
                }
            }
        }

        // 4) Bottom boundary (y = bottom)
        if dir.y != 0.0 {
            let t = (bottom - p0.y) / dir.y;
            if t >= 0.0 && t <= 1.0 {
                let x_at_t = p0.x + t * dir.x;
                if x_at_t >= left && x_at_t <= right {
                    t_candidates.push(t);
                }
            }
        }

        // Pick the smallest positive t (the first time we exit)
        let min_t = t_candidates
            .into_iter()
            .filter(|&t| t >= 0.0)
            .fold(f32::MAX, |a, b| a.min(b));

        // If we found a valid intersection:
        if min_t < f32::MAX {
            Some(p0 + dir * min_t)
        } else {
            None
        }
    }
}

impl CollidesWith<Self> for RectCollider {
    fn collides_with(&self, other: &Self) -> bool {
        let not_collided = other.tl.y < self.br().y
            || other.tl.x > self.br().x
            || other.br().y > self.tl.y
            || other.br().x < self.tl.x;

        !not_collided
    }

    fn resolution(&self, other: &Self) -> Vec2 {
        let self_br = self.br();
        let other_br = other.br();

        // Calculate overlap in both dimensions
        let x_overlap = (self_br.x.min(other_br.x) - self.tl.x.max(other.tl.x)).max(0.);
        let y_overlap = (self.tl.y.min(other.tl.y) - self_br.y.max(other_br.y)).max(0.);

        // Calculate the center points of both rectangles
        let self_center = self.center();
        let other_center = other.center();

        // If no overlap in either dimension, return zero
        if x_overlap == 0. || y_overlap == 0. {
            return Vec2::ZERO;
        }

        // let ratio = x_overlap / y_overlap;
        if x_overlap < y_overlap
        // || (0.75 < ratio && ratio < 1.25)
        {
            // Resolve horizontally
            let dir = (self_center.x - other_center.x).signum();
            Vec2::new(x_overlap * dir, 0.)
        } else {
            // Resolve vertically
            let dir = (self_center.y - other_center.y).signum();
            Vec2::new(0., y_overlap * dir)
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
pub struct CircleCollider {
    pub position: Vec2,
    pub radius: f32,
}

impl CircleCollider {
    pub fn expand(mut self, factor: f32) -> Self {
        self.radius *= factor;
        self
    }
}

impl CollidesWith<Self> for CircleCollider {
    fn collides_with(&self, other: &Self) -> bool {
        let distance = self.position.distance_squared(other.position);
        let combined_radii = self.radius + other.radius;
        distance <= combined_radii.powi(2)
    }

    fn resolution(&self, other: &Self) -> Vec2 {
        let diff = self.position - other.position;
        let distance_squared = diff.length_squared();
        let combined_radii = self.radius + other.radius;
        let combined_radii_squared = combined_radii * combined_radii;

        // If not overlapping, return zero vector
        if distance_squared >= combined_radii_squared {
            return Vec2::ZERO;
        }

        // Handle the case where circles are very close to same position
        const EPSILON: f32 = 0.0001;
        if distance_squared <= EPSILON {
            // Push to the right by combined radii
            return Vec2::new(combined_radii, 0.0);
        }

        let distance = distance_squared.sqrt();
        let overlap = combined_radii - distance;

        // Normalize diff without a separate division
        let direction = diff * (1.0 / distance);

        direction * (overlap + EPSILON)
    }
}

impl CollidesWith<RectCollider> for CircleCollider {
    fn collides_with(&self, other: &RectCollider) -> bool {
        let other_center = other.center();

        let dist_x = (self.position.x - other_center.x).abs();
        let dist_y = (self.position.y - other_center.y).abs();

        if dist_x > other.size.x * 0.5 + self.radius {
            return false;
        }

        if dist_y > other.size.y * 0.5 + self.radius {
            return false;
        }

        if dist_x <= other.size.x * 0.5 {
            return true;
        }

        if dist_y <= other.size.y * 0.5 {
            return true;
        }

        let corner_dist =
            (dist_x - other.size.x * 0.5).powi(2) + (dist_y - other.size.y * 0.5).powi(2);

        corner_dist <= self.radius.powi(2)
    }

    fn resolution(&self, other: &RectCollider) -> Vec2 {
        // Find the closest point on the rectangle to the circle's center
        let closest = Vec2::new(
            self.position.x.clamp(other.tl.x, other.tl.x + other.size.x),
            self.position.y.clamp(other.tl.y, other.tl.y + other.size.y),
        );

        let diff = self.position - closest;
        let distance = diff.length();

        // If not overlapping, return zero vector
        if distance >= self.radius {
            return Vec2::ZERO;
        }

        // Handle case where circle center is exactly on rectangle edge
        if distance == 0.0 {
            // Find which edge we're closest to and push out accordingly
            let to_left = self.position.x - other.tl.x;
            let to_right = (other.tl.x + other.size.x) - self.position.x;
            let to_top = self.position.y - other.tl.y;
            let to_bottom = (other.tl.y + other.size.y) - self.position.y;

            let min_dist = to_left.min(to_right).min(to_top).min(to_bottom);

            if min_dist == to_left {
                return Vec2::new(-self.radius, 0.0);
            }
            if min_dist == to_right {
                return Vec2::new(self.radius, 0.0);
            }
            if min_dist == to_top {
                return Vec2::new(0.0, -self.radius);
            }
            return Vec2::new(0.0, self.radius);
        }

        // Calculate the overlap and direction
        let overlap = self.radius - distance;
        let direction = diff / distance; // Normalized direction vector

        // Return the vector that will move the circle out of overlap
        direction * overlap
    }
}

impl CollidesWith<CircleCollider> for RectCollider {
    fn collides_with(&self, other: &CircleCollider) -> bool {
        other.collides_with(self)
    }

    fn resolution(&self, other: &CircleCollider) -> Vec2 {
        other.resolution(self)
    }
}

pub fn handle_collisions<T: Component>(
    map_query: Query<&SpatialHash, With<T>>,
    mut dynamic_bodies: Query<
        (
            &mut GlobalTransform,
            &mut Transform,
            &Collider,
            &mut Velocity,
            &mut TotalResolution,
            &mut Collision<T>,
        ),
        (With<DynamicBody>, With<layers::CollidesWith<T>>),
    >,
) {
    for (
        mut global_transform,
        mut transform,
        collider,
        mut velocity,
        mut resolution,
        mut entity_collision,
    ) in dynamic_bodies.iter_mut()
    {
        let original_collider = &collider;
        let mut global_t = global_transform.compute_transform();
        let mut collider = collider.absolute(&global_t);
        entity_collision.clear();

        for map in map_query.iter() {
            let mut colliders = map.nearby_objects(&collider.position()).collect::<Vec<_>>();

            colliders.sort_by(|d1, d2| {
                let d1 = collider.resolution(&d1.collider).length_squared();
                let d2 = collider.resolution(&d2.collider).length_squared();

                d2.partial_cmp(&d1).unwrap_or(Ordering::Equal)
            });

            for spatial::SpatialData {
                entity,
                collider: sc,
                ..
            } in colliders.into_iter()
            {
                if collider.collides_with(&sc) {
                    entity_collision.0.push(*entity);

                    let res = collider.resolution(&sc);
                    resolution.0 += res;

                    let res = res.extend(0.0);
                    transform.translation += res;
                    global_t.translation += res;
                    collider = original_collider.absolute(&global_t);

                    if res.y.abs() > 0. {
                        velocity.0.y = 0.;
                    }
                }
            }
        }

        // update global transform here so changes are observable in remaining collision systems.
        *global_transform = GlobalTransform::from(global_t);
    }
}

pub fn update_grounded<T: Component>(
    mut commands: Commands,
    map_query: Query<&SpatialHash, With<T>>,
    mut dynamic_bodies: Query<
        (Entity, &GlobalTransform, &Collider, &Velocity),
        (With<DynamicBody>, With<super::layers::CollidesWith<T>>),
    >,
) {
    for (entity, global_transform, collider, velocity) in dynamic_bodies.iter_mut() {
        let mut grounded = false;

        for map in map_query.iter() {
            let collider = collider.global_absolute(global_transform);
            let nearby_colliders = map.nearby_objects(&collider.position());

            for spatial::SpatialData {
                collider: static_collider,
                ..
            } in nearby_colliders.into_iter()
            {
                match (collider, static_collider) {
                    (AbsoluteCollider::Rect(a), AbsoluteCollider::Rect(b)) => {
                        let x_range = b.tl.x..b.br().x;

                        let on_top = (a.br().y - b.tl.y).abs() < 0.5;
                        let corner_inside =
                            x_range.contains(&a.tl.x) || x_range.contains(&a.br().x);
                        let no_going_up = velocity.0.y >= 0.;

                        if on_top && corner_inside && no_going_up {
                            grounded = true;
                            break;
                        }
                    }
                    _ => {
                        todo!("implement more grounded interactions")
                    }
                }
            }
        }

        if grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

pub fn update_brushing<T: Component>(
    mut commands: Commands,
    map_query: Query<&SpatialHash, With<T>>,
    mut dynamic_bodies: Query<
        (Entity, &GlobalTransform, &Collider, &Velocity),
        (With<DynamicBody>, With<super::layers::CollidesWith<T>>),
    >,
) {
    for (entity, global_transform, collider, velocity) in dynamic_bodies.iter_mut() {
        let mut left = false;
        let mut right = false;

        for map in map_query.iter() {
            let collider = collider.global_absolute(global_transform);
            let nearby_colliders = map.nearby_objects(&collider.position());

            for spatial::SpatialData {
                collider: static_collider,
                ..
            } in nearby_colliders
            {
                match (collider, static_collider) {
                    (AbsoluteCollider::Rect(a), AbsoluteCollider::Rect(b)) => {
                        let y_range = b.br().y..b.tl.y;

                        let corner_inside =
                            y_range.contains(&a.tl.y) || y_range.contains(&a.br().y);

                        // left
                        let adjacent = (a.tl.x - b.br().x).abs() < 0.1;
                        let no_going_right = velocity.0.x <= 0.;

                        if adjacent && corner_inside && no_going_right {
                            left = true;
                        }

                        // right
                        let adjacent = (a.br().x - b.tl.x).abs() < 0.1;
                        let no_going_left = velocity.0.x >= 0.;

                        if adjacent && corner_inside && no_going_left {
                            right = true;
                        }
                    }
                    _ => {
                        todo!("implement more grounded interactions")
                    }
                }
            }
        }

        if left {
            commands.entity(entity).insert(BrushingLeft);
        } else {
            commands.entity(entity).remove::<BrushingLeft>();
        }

        if right {
            commands.entity(entity).insert(BrushingRight);
        } else {
            commands.entity(entity).remove::<BrushingRight>();
        }
    }
}

pub fn handle_dynamic_body_collsions<T: Component>(
    mut dynamic_bodies: Query<
        (
            Entity,
            &mut Transform,
            &Collider,
            Option<&Massive>,
            &mut TotalResolution,
        ),
        (With<DynamicBody>, With<super::layers::CollidesWith<T>>),
    >,
    other_bodies: Query<
        (Entity, &Transform, &Collider, Option<&Massive>),
        (
            With<DynamicBody>,
            With<T>,
            Without<super::layers::CollidesWith<T>>,
        ),
    >,
) {
    let mut other_bodies = other_bodies.iter().collect::<Vec<_>>();
    other_bodies.sort_by_key(|(_, _, _, m)| {
        if m.is_some() {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    let mut spatial = spatial::SpatialHash::new(32.);

    for (entity, transform, collider, massive) in other_bodies.iter() {
        let absolute = collider.absolute(transform);
        spatial.insert(spatial::SpatialData {
            collider: absolute,
            data: (massive.cloned(), *collider),
            entity: *entity,
        });
    }

    for (entity, mut transform, collider, massive, mut resolution) in dynamic_bodies.iter_mut() {
        let original_collider = &collider;
        let mut collider = collider.absolute(&transform);

        let mut update_active = false;

        // TODO: this shit is awful
        //
        // For some reason god forsaken, this will update twice even though the position in the hash is updated
        // before the other, overlapping entity updates itself.
        for spatial::SpatialData {
            entity: se,
            collider: sc,
            ..
        } in spatial.nearby_objects(&collider.position())
        {
            if collider.collides_with(sc) && massive.is_none() {
                let res_v = collider.resolution(sc);
                resolution.0 += res_v;
                transform.translation += Vec3::new(res_v.x, res_v.y, 0.);
                collider = original_collider.absolute(&transform);
                update_active = true;
            }
        }

        if update_active {
            for spatial::SpatialData {
                entity: se,
                collider: sc,
                ..
            } in spatial.objects_in_cell_mut(&collider.position())
            {
                if *se == entity {
                    *sc = collider;
                    break;
                }
            }
        }
    }
}

/// A marker component that indicates a tile should
/// generate collision when inserted.
#[derive(Debug, Default, Component)]
pub struct TilesetCollider;

/// Sets the tile size, used primarily for collider construction.
#[derive(Debug, Resource)]
pub struct TilesetSize(pub f32);

impl Default for TilesetSize {
    fn default() -> Self {
        Self(16.0)
    }
}

pub fn build_tile_set_colliders(
    mut commands: Commands,
    tiles: Query<(&Transform, &Parent), Added<TilesetCollider>>,
    levels: Query<Entity>,
    size: Res<TilesetSize>,
    // manual_collision: Query<&Transform, Added<annual::Collision>>,
) {
    //let mut num_colliders = 0;

    // ~14k without combining
    // ~600 with horizontal combining

    if tiles.is_empty() {
        return;
    }

    // WARN: assumes that one level is loaded at a time!!
    let level = tiles
        .iter()
        .next()
        .map(|(_, p)| levels.get(p.get()).unwrap())
        .unwrap();

    let mut parents = HashSet::<Entity>::default();
    for (_, parent) in tiles.iter() {
        parents.insert(parent.get());
    }

    let tile_size = size.0;
    let offset = tile_size / 2.;

    for parent in parents.into_iter() {
        let cached_collider_positions = tiles
            .iter()
            .filter(|(_, p)| p.get() == parent)
            .map(|(t, _)| Vec2::new(t.translation.x + offset, t.translation.y + offset))
            .collect::<Vec<_>>();

        if cached_collider_positions.is_empty() {
            return;
        }

        commands.entity(parent).with_children(|level| {
            for (pos, collider) in
                build_colliders_from_vec2(cached_collider_positions, tile_size).into_iter()
            {
                level.spawn((
                    Transform::from_translation((pos - Vec2::splat(tile_size / 2.)).extend(0.)),
                    StaticBody,
                    collider,
                ));
                //num_colliders += 1;
            }
        });
    }

    //println!("num_colliders: {num_colliders}");
}

fn build_colliders_from_vec2(mut positions: Vec<Vec2>, tile_size: f32) -> Vec<(Vec2, Collider)> {
    positions.sort_by(|a, b| {
        let y_cmp = a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp == std::cmp::Ordering::Equal {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_cmp
        }
    });

    let mut rows = Vec::with_capacity(positions.len() / 2);
    let mut current_y = None;
    let mut current_xs = Vec::with_capacity(positions.len() / 2);
    for v in positions.into_iter() {
        match current_y {
            None => {
                current_y = Some(v.y);
                current_xs.push(v.x);
            }
            Some(y) => {
                if v.y == y {
                    current_xs.push(v.x);
                } else {
                    rows.push((y, current_xs.clone()));
                    current_xs.clear();

                    current_y = Some(v.y);
                    current_xs.push(v.x);
                }
            }
        }
    }

    match current_y {
        Some(y) => {
            rows.push((y, current_xs));
        }
        None => unreachable!(),
    }

    #[derive(Debug, Clone, Copy)]
    struct Plate {
        y_start: f32,
        y_end: f32,
        x_start: f32,
        x_end: f32,
    }

    let mut row_plates = Vec::with_capacity(rows.len());
    for (y, row) in rows.into_iter() {
        let mut current_x = None;
        let mut x_start = None;
        let mut plates = Vec::with_capacity(row.len() / 4);

        for x in row.iter() {
            match (current_x, x_start) {
                (None, None) => {
                    current_x = Some(*x);
                    x_start = Some(*x);
                }
                (Some(cx), Some(xs)) => {
                    if *x > cx + tile_size {
                        plates.push(Plate {
                            x_end: cx + tile_size,
                            x_start: xs,
                            y_start: y - tile_size,
                            y_end: y,
                        });
                        x_start = Some(*x);
                    }

                    current_x = Some(*x);
                }
                _ => unreachable!(),
            }
        }

        match (current_x, x_start) {
            (Some(cx), Some(xs)) => {
                plates.push(Plate {
                    x_end: cx + tile_size,
                    x_start: xs,
                    y_start: y - tile_size,
                    y_end: y,
                });
            }
            _ => unreachable!(),
        }

        row_plates.push(plates);
    }

    let mut output = HashMap::<(i32, i32), Vec<Plate>>::default();
    for plates in row_plates.iter() {
        for plate in plates.iter() {
            let entry = output
                .entry((plate.x_start as i32, plate.x_end as i32))
                .or_default();
            entry.push(*plate);
        }
    }

    for (_, plates) in output.iter_mut() {
        let mut new_plates = Vec::with_capacity(plates.len());

        while let Some(mut plate) = plates.pop() {
            while let Some(next_plate) = plates.pop() {
                if next_plate.y_end == plate.y_start {
                    let end = plate.y_end;
                    plate = next_plate;
                    plate.y_end = end;
                } else {
                    new_plates.push(next_plate);
                    break;
                }
            }

            new_plates.push(plate);
        }

        *plates = new_plates;
    }

    let mut colliders = Vec::new();
    for plate in output.into_values().flatten() {
        colliders.push((
            Vec2::new(plate.x_start, plate.y_end),
            Collider::from_rect(
                Vec2::ZERO,
                Vec2::new(plate.x_end - plate.x_start, plate.y_end - plate.y_start),
            ),
        ));
    }

    colliders
}
