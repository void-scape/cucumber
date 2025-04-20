use std::time::Duration;

use crate::{
    HEIGHT, WIDTH, assets,
    auto_collider::AutoCollider,
    bullet::{BulletTimer, BulletType},
    health::{Damage, Dead, Health, HealthSet},
};
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use physics::{Physics, layers::TriggersWith, prelude::*};
use rand::Rng;

pub const GLOBAL_ENEMY_SPEED: f32 = 4.;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, shoot_bullets)
            .add_systems(Physics, handle_death.after(HealthSet));
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    let mut rng = rand::rng();
    let padding = 20.;

    for _ in 0..5 {
        let x = rng.random_range((-WIDTH / 2. + padding)..WIDTH / 2. - padding);
        spawn_enemy(
            &mut commands,
            &server,
            Enemy::Basic,
            Transform::from_xyz(x, HEIGHT / 2. + 4., 0.),
        );
    }
}

fn spawn_enemy(commands: &mut Commands, server: &AssetServer, enemy: Enemy, bundle: impl Bundle) {
    let mut sprite = enemy.sprite(server);
    sprite.flip_y = true;
    commands.spawn((
        enemy,
        Velocity(Vec2::NEG_Y * GLOBAL_ENEMY_SPEED * enemy.speed_mul()),
        sprite,
        bundle,
        Health::full(1),
        BulletTimer {
            timer: Timer::new(Duration::from_millis(1500), TimerMode::Repeating),
        },
    ));
}

#[derive(Clone, Copy, Component)]
#[require(Transform, Velocity, Visibility, layers::Enemy, AutoCollider)]
enum Enemy {
    Basic,
}

impl Enemy {
    pub fn speed_mul(&self) -> f32 {
        match self {
            Self::Basic => 1.,
        }
    }

    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Basic => assets::sprite_rect(server, assets::SHIPS_PATH, Vec2::new(1., 1.)),
        }
    }
}

fn handle_death(q: Query<(Entity, &Enemy), With<Dead>>, mut commands: Commands) {
    for (entity, _enemy) in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn shoot_bullets(
    mut enemies: Query<(&mut BulletTimer, &Transform), With<Enemy>>,
    time: Res<Time>,
    server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (mut timer, transform) in enemies.iter_mut() {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            let mut new_transform = transform.clone();
            new_transform.translation.y -= 8.0;

            commands.spawn((
                BulletType::Basic,
                Velocity(Vec2::new(0.0, -200.)),
                new_transform,
                TriggersWith::<layers::Player>::default(),
                Damage::new(1),
            ));

            commands
                .spawn((
                    SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                    PlaybackSettings {
                        volume: Volume::Decibels(-24.0),
                        ..PlaybackSettings::ONCE
                    },
                ))
                .effect(BandPassNode::new(1000.0, 4.0));
        }
    }
}
