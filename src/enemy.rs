use bevy::prelude::*;
use bevy::sprite::Anchor;
use physics::prelude::*;
use rand::Rng;

use crate::{HEIGHT, WIDTH};

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
    commands
        .spawn((
            enemy,
            Velocity(Vec2::NEG_Y * GLOBAL_ENEMY_SPEED * enemy.speed_mul()),
            bundle,
        ))
        .with_children(|root| {
            root.spawn(Sprite {
                image: server.load("invaders_sprites.png"),
                rect: Some(Rect::from_corners(
                    Vec2::X * 2. * 8.,
                    Vec2::X * 2.5 * 8. + Vec2::Y * 8.,
                )),
                anchor: Anchor::CenterRight,
                ..Default::default()
            });
            root.spawn(Sprite {
                image: server.load("invaders_sprites.png"),
                rect: Some(Rect::from_corners(
                    Vec2::X * 2. * 8.,
                    Vec2::X * 2.5 * 8. + Vec2::Y * 8.,
                )),
                flip_x: true,
                anchor: Anchor::CenterLeft,
                ..Default::default()
            });
        });
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
}
