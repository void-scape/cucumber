use crate::{
    HEIGHT, assets,
    auto_collider::ImageCollider,
    bullet::{BulletTimer, BulletType},
    health::{Damage, Dead, Health, HealthSet},
};
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use physics::{Physics, layers::TriggersWith, prelude::*};
use std::time::Duration;

pub const GLOBAL_ENEMY_SPEED: f32 = 4.;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (spawn_formations, shoot_bullets))
            .add_systems(Physics, handle_death.after(HealthSet));
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Formation::Triangle);

    //let mut rng = rand::rng();
    //let padding = 20.;
    //
    //for _ in 0..5 {
    //    let x = rng.random_range((-WIDTH / 2. + padding)..WIDTH / 2. - padding);
    //    Enemy::Common.spawn_with(
    //        &mut commands,
    //        &server,
    //        Transform::from_xyz(x, HEIGHT / 2. + 4., 0.),
    //    );
    //}
}

#[derive(Component)]
#[require(Transform, Visibility)]
enum Formation {
    Triangle,
}

impl Formation {
    pub fn enemies(&self) -> &'static [(Enemy, Vec2)] {
        match self {
            Self::Triangle => {
                const {
                    &[
                        (Enemy::Common, Vec2::new(-20., -40.)),
                        (Enemy::Common, Vec2::ZERO),
                        (Enemy::Common, Vec2::new(20., -40.)),
                    ]
                }
            }
        }
    }

    pub fn lowest_y(&self) -> f32 {
        debug_assert!(!self.enemies().is_empty());
        self.enemies()
            .iter()
            .map(|(_, pos)| pos.y)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap()
    }
}

const LARGEST_SPRITE_SIZE: f32 = 16.;

fn spawn_formations(
    mut commands: Commands,
    server: Res<AssetServer>,
    formations: Query<(Entity, &Formation), Without<Children>>,
) {
    for (root, formation) in formations.iter() {
        // bottom of formation spawns immediately above the top of the screen
        let y = HEIGHT / 2. - formation.lowest_y() + LARGEST_SPRITE_SIZE / 2.;
        commands.entity(root).insert(Transform::from_xyz(0., y, 0.));

        for (enemy, position) in formation.enemies().iter() {
            enemy.spawn_child_with(
                root,
                &mut commands,
                &server,
                Transform::from_translation(position.extend(0.)),
            );
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Transform, Velocity, Visibility, layers::Enemy, ImageCollider)]
enum Enemy {
    Common,
}

impl Enemy {
    pub fn spawn_with(&self, commands: &mut Commands, server: &AssetServer, bundle: impl Bundle) {
        let mut entity_commands = commands.spawn_empty();
        self.insert(&mut entity_commands, server, bundle);
    }

    pub fn spawn_child_with(
        &self,
        entity: Entity,
        commands: &mut Commands,
        server: &AssetServer,
        bundle: impl Bundle,
    ) {
        let mut entity_commands = commands.spawn_empty();
        self.insert(&mut entity_commands, server, bundle);
        let id = entity_commands.id();
        commands.entity(entity).add_child(id);
    }

    fn insert(&self, commands: &mut EntityCommands, server: &AssetServer, bundle: impl Bundle) {
        commands.insert((
            *self,
            self.health(),
            self.sprite(server),
            self.bullets(),
            Velocity(Vec2::NEG_Y * GLOBAL_ENEMY_SPEED * self.speed_mul()),
            bundle,
        ));
    }

    pub fn health(&self) -> Health {
        match self {
            Self::Common => Health::full(1),
        }
    }

    pub fn speed_mul(&self) -> f32 {
        match self {
            Self::Common => 1.,
        }
    }

    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Common => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(2, 3)),
        }
    }

    pub fn bullets(&self) -> BulletTimer {
        match self {
            Self::Common => BulletTimer {
                timer: Timer::new(Duration::from_millis(1500), TimerMode::Repeating),
            },
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
                BulletType::Common,
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
