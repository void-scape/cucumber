use std::time::Duration;

use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use physics::{Physics, prelude::*};

use crate::health::{Damage, Health, HealthSet};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Physics,
            (handle_enemy_collision, handle_player_collision).before(HealthSet),
        )
        .add_systems(Update, manage_lifetime);
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
#[require(Velocity, DynamicBody, Lifetime)]
pub struct Bullet;

#[derive(Clone, Copy, Component)]
#[require(Bullet)]
#[component(on_add = Self::on_add)]
pub enum BulletType {
    Basic,
}

impl BulletType {
    fn on_add(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let ty = *world.get::<BulletType>(entity).unwrap();

        match ty {
            BulletType::Basic => {
                world.commands().entity(entity).insert(BasicBullet);
            }
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Collider(basic_collider))]
#[component(on_add = on_add_basic)]
pub struct BasicBullet;

fn basic_collider() -> Collider {
    let size = Vec2::new(1.0, 1.0) * crate::RESOLUTION_SCALE;
    Collider::from_rect(Vec2::new(-size.x / 2.0, size.y / 2.0), size)
}

fn on_add_basic(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let server = world.get_resource::<AssetServer>().unwrap();

    let bullet = server.load("sprites/bullet.png");

    world
        .commands()
        .entity(entity)
        .insert(Sprite::from_image(bullet));
}

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
