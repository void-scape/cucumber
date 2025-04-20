use crate::{
    assets,
    health::{Damage, Health, HealthSet},
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use physics::{Physics, prelude::*};
use std::time::Duration;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Physics,
            (handle_enemy_collision, handle_player_collision).before(HealthSet),
        )
        .add_systems(Update, manage_lifetime)
        .add_systems(PostUpdate, init_spawned_bullets);
    }
}

#[derive(Default, Component)]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    #[default]
    South,
    SouthWest,
    West,
}

impl Direction {
    pub fn velocity(&self) -> Velocity {
        match self {
            Self::NorthWest => Velocity(Vec2::new(-1., 1.)),
            Self::North => Velocity(Vec2::new(0., 1.)),
            Self::NorthEast => Velocity(Vec2::new(1., 1.)),
            Self::East => Velocity(Vec2::new(1., 0.)),
            Self::SouthEast => Velocity(Vec2::new(1., -1.)),
            Self::South => Velocity(Vec2::new(0., -1.)),
            Self::SouthWest => Velocity(Vec2::new(-1., -1.)),
            Self::West => Velocity(Vec2::new(-1., 0.)),
        }
    }
}

#[derive(Component)]
pub struct BulletSpeed(pub f32);

impl Default for BulletSpeed {
    fn default() -> Self {
        Self(200.)
    }
}

fn init_spawned_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &BulletSpeed, &Direction, &BulletSprite), Without<Velocity>>,
    server: Res<AssetServer>,
) {
    for (entity, speed, direction, sprite) in bullets.iter() {
        let mut velocity = direction.velocity();
        velocity.0 *= speed.0;
        let mut sprite = assets::sprite_rect(&server, sprite.path, sprite.cell);
        sprite.flip_y = velocity.0.y < 0.;
        sprite.flip_x = velocity.0.x < 0.;
        commands.entity(entity).insert((velocity, sprite));
    }
}

#[derive(Component)]
pub struct BulletTimer {
    pub timer: Timer,
}

#[derive(Debug, Component)]
pub struct Lifetime(pub Timer);

impl Default for Lifetime {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(2), TimerMode::Once))
    }
}

fn manage_lifetime(mut q: Query<(Entity, &mut Lifetime)>, time: Res<Time>, mut commands: Commands) {
    let delta = time.delta();

    for (entity, mut lifetime) in q.iter_mut() {
        lifetime.0.tick(delta);

        if lifetime.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Clone, Copy, Component, Default)]
#[require(Direction, BulletSpeed, DynamicBody, Lifetime)]
pub struct Bullet;

#[derive(Clone, Copy, Component)]
#[require(Bullet)]
#[component(on_add = Self::on_add)]
pub enum BulletType {
    Basic,
    Common,
}

impl BulletType {
    fn on_add(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let ty = *world.get::<BulletType>(entity).unwrap();

        match ty {
            BulletType::Basic => {
                world.commands().entity(entity).insert(BasicBullet);
            }
            BulletType::Common => {
                world.commands().entity(entity).insert(CommonBullet);
            }
        }
    }
}

#[derive(Component)]
pub struct BulletSprite {
    path: &'static str,
    cell: Vec2,
}

impl BulletSprite {
    pub fn from_cell(x: usize, y: usize) -> Self {
        Self {
            path: assets::PROJECTILES_PATH,
            cell: Vec2::new(x as f32, y as f32),
        }
    }
}

fn small_collider() -> Collider {
    let size = Vec2::new(1.0, 1.0) * crate::RESOLUTION_SCALE;
    Collider::from_rect(Vec2::new(-size.x / 2.0, size.y / 2.0), size)
}

#[derive(Clone, Copy, Component)]
#[require(Collider(small_collider), BulletSprite(|| BulletSprite::from_cell(0, 0)))]
pub struct BasicBullet;

#[derive(Clone, Copy, Component)]
#[require(Collider(small_collider), BulletSprite(|| BulletSprite::from_cell(4, 0)))]
pub struct CommonBullet;

fn handle_enemy_collision(
    bullets: Query<(Entity, &Damage, &Triggers<layers::Enemy>), With<Bullet>>,
    mut enemies: Query<&mut Health>,
    mut commands: Commands,
) {
    for (bullet, damage, collision) in bullets.iter() {
        let Some(first) = collision.entities().first() else {
            continue;
        };

        if let Ok(mut enemy) = enemies.get_mut(*first) {
            enemy.damage(**damage);
            commands.entity(bullet).despawn();
        }
    }
}

fn handle_player_collision(
    bullets: Query<(Entity, &Damage, &Triggers<layers::Player>), With<Bullet>>,
    mut player: Query<&mut Health>,
    mut commands: Commands,
) {
    for (bullet, damage, collision) in bullets.iter() {
        let Some(first) = collision.entities().first() else {
            continue;
        };

        if let Ok(mut player) = player.get_mut(*first) {
            player.damage(**damage);
            commands.entity(bullet).despawn();
        }
    }
}
