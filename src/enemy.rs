use crate::{
    HEIGHT, assets,
    auto_collider::ImageCollider,
    bullet::{BulletRate, BulletSpeed, BulletTimer, emitter::SoloEmitter},
    health::{Dead, Health, HealthSet},
};
use bevy::prelude::*;
use physics::{Physics, prelude::*};
use std::time::Duration;

pub const GLOBAL_ENEMY_SPEED: f32 = 4.;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (spawn_formations,))
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
                        (Enemy::Common, Vec2::new(-40., -40.)),
                        (Enemy::Common, Vec2::ZERO),
                        (Enemy::Common, Vec2::new(40., -40.)),
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
        self.insert_emitter(&mut entity_commands);
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
        self.insert_emitter(&mut entity_commands);
        let id = entity_commands.id();
        commands.entity(entity).add_child(id);
    }

    fn insert_emitter(&self, commands: &mut EntityCommands) {
        commands.with_child((SoloEmitter::<layers::Player>::new(),));
    }

    fn insert(&self, commands: &mut EntityCommands, server: &AssetServer, bundle: impl Bundle) {
        commands.insert((
            *self,
            self.health(),
            self.sprite(server),
            self.bullets(),
            BulletRate(0.20),
            BulletSpeed(0.5),
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
