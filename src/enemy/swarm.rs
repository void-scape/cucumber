use std::f32::consts::PI;
use std::time::Duration;

use super::Enemy;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use super::timeline::ENEMY_Z;
use crate::WIDTH;
use crate::bullet::emitter::EmitterDelay;
use crate::{
    Layer,
    auto_collider::ImageCollider,
    bullet::emitter::{BulletModifiers, Rate, SwarmEmitter},
    effects::Explosion,
    health::Health,
    sprites::CellSprite,
};
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tween::combinator::sequence;
use bevy_tween::combinator::tween;
use bevy_tween::prelude::AnimationBuilderExt;
use bevy_tween::prelude::EaseKind;
use bevy_tween::tween::IntoTarget;
use physics::linear_velocity;

pub const SWARM_SPEED: f32 = 60.;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(1.),
    CellSprite::new8("ships.png", UVec2::new(3, 0)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    SwarmEmitter,
    BulletModifiers {
        rate: Rate::Factor(0.2),
        ..Default::default()
    },
    Trauma::NONE,
    Explosion::Small,
)]
pub struct Swarm;

#[derive(Clone, Copy)]
pub struct SwarmFormation {
    pub start: Vec2,
    pub anchor: SwarmAnchor,
    pub heading: SwarmHeading,
    pub count: usize,
    pub gap: f32,
    // strength of the random position noise [0.0..]
    pub noise: Vec2,
}

impl Default for SwarmFormation {
    fn default() -> Self {
        Self {
            start: Vec2::ZERO,
            anchor: SwarmAnchor::Center,
            heading: SwarmHeading::Linear(Vec2::ZERO),
            count: 5,
            gap: 15.,
            noise: Vec2::new(5., 20.),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SwarmAnchor {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy)]
pub enum SwarmHeading {
    Linear(Vec2),
}

pub fn swarm(swarm: SwarmFormation) -> Formation {
    Formation::with_velocity(Vec2::ZERO, move |formation: &mut EntityCommands, _| {
        let center = swarm.start;
        formation.with_children(|root| {
            for i in 0..swarm.count {
                let anchor_offset = match swarm.anchor {
                    SwarmAnchor::Center => -(swarm.count as f32 / 2.) * swarm.gap,
                    SwarmAnchor::Left => -(swarm.count as f32 * swarm.gap) - 8.,
                    SwarmAnchor::Right => 8.,
                };

                let x = i as f32 * swarm.gap + anchor_offset;
                let y = noise::simplex_noise_2d(Vec2::new(x + center.x, 0.)) * swarm.noise.y;
                let x_noise = noise::simplex_noise_2d(Vec2::new(x + center.y, y)) * swarm.noise.x;

                match swarm.heading {
                    SwarmHeading::Linear(velocity) => {
                        root.spawn((
                            Swarm,
                            Platoon(root.target_entity()),
                            EmitterDelay::new(0.2 * i as f32),
                            Transform::from_xyz(x + x_noise + center.x, y + center.y, 0.),
                            LinearVelocity(velocity),
                        ));
                    }
                }
            }
        });
    })
}

pub fn left_swing() -> Formation {
    Formation::with_velocity(Vec2::ZERO, move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            for i in 0..4 {
                //let y = noise::simplex_noise_2d(Vec2::new(x + center.x, 0.)) * swarm.noise.y;
                //let x_noise = noise::simplex_noise_2d(Vec2::new(x + center.y, y)) * swarm.noise.x;

                //match swarm.heading {
                //SwarmHeading::Linear(velocity) => {
                let enemy = root
                    .spawn((
                        Swarm,
                        Platoon(root.target_entity()),
                        EmitterDelay::new(0.2 * i as f32),
                        Transform::from_xyz(-crate::WIDTH / 2. - 4., 0., ENEMY_Z),
                        //Transform::from_xyz(x + x_noise + center.x, y + center.y, 0.),
                        //LinearVelocity(velocity),
                        LinearVelocity::default(),
                    ))
                    .id();
                root.commands().entity(enemy).animation().insert(sequence((
                    tween(
                        Duration::from_secs_f32(i as f32 * 12. / SWARM_SPEED),
                        EaseKind::Linear,
                        enemy
                            .into_target()
                            .with(linear_velocity(Vec2::ZERO, Vec2::ZERO)),
                    ),
                    tween(
                        Duration::from_secs_f32(1. + i as f32 * 0.8),
                        EaseKind::Linear,
                        enemy.into_target().with(linear_velocity(
                            Vec2::from_angle(-PI / 6.).normalize() * SWARM_SPEED,
                            Vec2::NEG_Y * SWARM_SPEED,
                        )),
                    ),
                )));

                //}
                //}
            }
        });
    })
}

pub fn swing() -> Formation {
    Formation::with_velocity(Vec2::ZERO, move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            for i in 0..4 {
                //let y = noise::simplex_noise_2d(Vec2::new(x + center.x, 0.)) * swarm.noise.y;
                //let x_noise = noise::simplex_noise_2d(Vec2::new(x + center.y, y)) * swarm.noise.x;

                //match swarm.heading {
                //SwarmHeading::Linear(velocity) => {
                let enemy = root
                    .spawn((
                        Swarm,
                        Platoon(root.target_entity()),
                        EmitterDelay::new(0.2 * i as f32),
                        Transform::from_xyz(-crate::WIDTH / 2. - 4., 0., ENEMY_Z),
                        //Transform::from_xyz(x + x_noise + center.x, y + center.y, 0.),
                        //LinearVelocity(velocity),
                        LinearVelocity::default(),
                    ))
                    .id();
                root.commands().entity(enemy).animation().insert(sequence((
                    tween(
                        Duration::from_secs_f32(i as f32 * 12. / SWARM_SPEED),
                        EaseKind::Linear,
                        enemy
                            .into_target()
                            .with(linear_velocity(Vec2::ZERO, Vec2::ZERO)),
                    ),
                    tween(
                        Duration::from_secs_f32(1. + i as f32 * 0.8),
                        EaseKind::Linear,
                        enemy.into_target().with(linear_velocity(
                            Vec2::from_angle(-PI / 6.).normalize() * SWARM_SPEED,
                            Vec2::NEG_Y * SWARM_SPEED,
                        )),
                    ),
                )));

                //}
                //}
            }
        });
    })
}

const SWARM_THRESHOLD: f32 = WIDTH * 1.25;

#[derive(Debug, Component)]
pub enum SwarmMovement {
    Left,
    Right,
}

pub(super) fn swarm_movement(
    mut q: Query<(&mut LinearVelocity, &GlobalTransform, &SwarmMovement)>,
) {
    for (mut velocity, transform, movement) in &mut q {
        let translation = transform.compute_transform().translation;

        match movement {
            SwarmMovement::Left => {
                if translation.x > SWARM_THRESHOLD && velocity.x > 0. {
                    velocity.x = -SWARM_SPEED;
                } else if translation.x < -SWARM_THRESHOLD && velocity.x < 0. {
                    velocity.x = SWARM_SPEED;
                } else if velocity.x == 0. {
                    velocity.x = SWARM_SPEED;
                }
            }
            SwarmMovement::Right => {
                if translation.x < SWARM_THRESHOLD && velocity.x < 0. {
                    velocity.x = -SWARM_SPEED;
                } else if translation.x > -SWARM_THRESHOLD && velocity.x > 0. {
                    velocity.x = SWARM_SPEED;
                } else if velocity.x == 0. {
                    velocity.x = SWARM_SPEED;
                }
            }
        }

        if (-WIDTH * 0.5..WIDTH * 0.5).contains(&translation.x) {
            let proportion = (translation.x + WIDTH * 0.5) / WIDTH;

            let proportion = if velocity.x > 0. {
                proportion
            } else {
                1. - proportion
            };

            let sin = ((proportion * core::f32::consts::PI).cos() - 0.5) * -10.;
            velocity.y = sin;
        } else {
            velocity.y = 0.;
        }
    }
}
