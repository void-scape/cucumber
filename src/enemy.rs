use crate::{HEIGHT, WIDTH};
use bevy::prelude::*;
use physics::prelude::*;
use rand::Rng;

pub const GLOBAL_ENEMY_SPEED: f32 = 4.;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
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
    commands.spawn((
        enemy,
        Velocity(Vec2::NEG_Y * GLOBAL_ENEMY_SPEED * enemy.speed_mul()),
        bundle,
        Sprite {
            image: server.load("invaders_sprites.png"),
            rect: Some(enemy.sprite_rect()),
            ..Default::default()
        },
    ));
}

#[derive(Clone, Copy, Component)]
#[require(Transform, Velocity, Visibility)]
enum Enemy {
    Basic,
}

impl Enemy {
    pub fn speed_mul(&self) -> f32 {
        match self {
            Self::Basic => 1.,
        }
    }

    pub fn sprite_rect(&self) -> Rect {
        match self {
            Self::Basic => Rect::from_corners(Vec2::X * 3. * 8., Vec2::X * 4. * 8. + Vec2::Y * 8.),
        }
    }
}
