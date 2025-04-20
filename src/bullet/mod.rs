use crate::{
    assets,
    auto_collider::ImageCollider,
    health::{Damage, Health, HealthSet},
};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use physics::{Physics, prelude::*};
use std::time::Duration;

pub mod emitter;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum BulletSystems {
    Collision,
    Lifetime,
    Velocity,
    Sprite,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(emitter::EmitterPlugin)
            .add_systems(
                Physics,
                (handle_enemy_collision, handle_player_collision)
                    .before(HealthSet)
                    .in_set(BulletSystems::Collision),
            )
            .add_systems(Update, manage_lifetime.in_set(BulletSystems::Lifetime))
            .add_systems(
                PostUpdate,
                (
                    init_bullet_velocity.in_set(BulletSystems::Velocity),
                    init_bullet_sprite.in_set(BulletSystems::Sprite),
                )
                    .chain(),
            );
    }
}

#[derive(Default, Component)]
pub enum Polarity {
    North,
    #[default]
    South,
}

impl Polarity {
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Self::North => Vec2::Y,
            Self::South => Vec2::NEG_Y,
        }
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
    pub fn to_vec2(self) -> Vec2 {
        match self {
            Self::NorthWest => Vec2::new(-1., 1.).normalize(),
            Self::North => Vec2::Y,
            Self::NorthEast => Vec2::ONE.normalize(),
            Self::East => Vec2::X,
            Self::SouthEast => Vec2::new(1., -1.).normalize(),
            Self::South => Vec2::NEG_Y,
            Self::SouthWest => Vec2::NEG_ONE.normalize(),
            Self::West => Vec2::NEG_X,
        }
    }
}
/// The rate at which bullets should fire.
///
/// This doesn't have any particular unit;
/// emitters can interpret this however they like.
#[derive(Component)]
pub struct BulletRate(pub f32);

impl Default for BulletRate {
    fn default() -> Self {
        Self(1.0)
    }
}

/// The speed at which bullets should travel.
///
/// This doesn't have any particular unit;
/// emitters can interpret this however they like.
#[derive(Component)]
pub struct BulletSpeed(pub f32);

impl Default for BulletSpeed {
    fn default() -> Self {
        Self(200.0)
    }
}

fn init_bullet_velocity(
    mut commands: Commands,
    bullets: Query<(Entity, &BulletSpeed, &Polarity), Without<Velocity>>,
) {
    for (entity, speed, polarity) in bullets.iter() {
        let velocity = Velocity(polarity.to_vec2() * speed.0);
        commands.entity(entity).insert(velocity);
    }
}

fn init_bullet_sprite(
    mut commands: Commands,
    bullets: Query<(Entity, &BulletSprite, &Velocity), Without<Sprite>>,
    server: Res<AssetServer>,
) {
    for (entity, sprite, velocity) in bullets.iter() {
        let mut sprite = assets::sprite_rect8(&server, sprite.path, sprite.cell);
        sprite.flip_y = velocity.0.y < 0.;
        sprite.flip_x = velocity.0.x < 0.;
        commands.entity(entity).insert(sprite);
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
#[require(BulletSpeed, DynamicBody, Lifetime)]
pub struct Bullet;

#[derive(Clone, Copy, Component)]
#[require(Bullet, Polarity)]
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
#[require(ImageCollider)]
pub struct BulletSprite {
    path: &'static str,
    cell: UVec2,
}

impl BulletSprite {
    pub fn from_cell(x: u32, y: u32) -> Self {
        Self {
            path: assets::PROJECTILES_PATH,
            cell: UVec2::new(x, y),
        }
    }
}

fn small_collider() -> Collider {
    let size = Vec2::new(1.0, 1.0) * crate::RESOLUTION_SCALE;
    Collider::from_rect(Vec2::new(-size.x / 2.0, size.y / 2.0), size)
}

#[derive(Clone, Copy, Component)]
#[require(BulletSprite(|| BulletSprite::from_cell(0, 1)))]
pub struct BasicBullet;

#[derive(Clone, Copy, Component)]
#[require(BulletSprite(|| BulletSprite::from_cell(2, 1)))]
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
