use super::Enemy;
use super::FaceVelocity;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use super::timeline::ENEMY_Z;
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
use std::f32::consts::PI;
use std::time::Duration;

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
    FaceVelocity,
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

pub fn three() -> Formation {
    Formation::with_velocity(Vec2::ZERO, move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            for i in 0..3 {
                let x = -(i as f32 - 2. / 2.) * 24.;
                let enemy = root
                    .spawn((
                        Swarm,
                        Platoon(root.target_entity()),
                        EmitterDelay::new(i as f32 * 12. * 4. / SWARM_SPEED + 0.1),
                        Transform::from_xyz(x, 4., ENEMY_Z),
                    ))
                    .id();
                root.commands().entity(enemy).animation().insert(sequence((
                    tween(
                        Duration::from_secs_f32(i as f32 * 12. * 4. / SWARM_SPEED),
                        EaseKind::Linear,
                        enemy
                            .into_target()
                            .with(linear_velocity(Vec2::ZERO, Vec2::ZERO)),
                    ),
                    tween(
                        Duration::from_secs_f32(1. + i as f32 * 0.3),
                        EaseKind::Linear,
                        enemy
                            .into_target()
                            .with(linear_velocity(Vec2::NEG_Y * SWARM_SPEED * 1.5, Vec2::ZERO)),
                    ),
                    tween(
                        Duration::from_secs_f32(2.),
                        EaseKind::Linear,
                        enemy.into_target().with(linear_velocity(
                            Vec2::ZERO,
                            Vec2::new(x / 12., -1.).normalize() * SWARM_SPEED * 3.,
                        )),
                    ),
                )));
            }
        });
    })
}

pub fn right_swing() -> Formation {
    swing(Swing::Right)
}

pub fn left_swing() -> Formation {
    swing(Swing::Left)
}

enum Swing {
    Left,
    Right,
}

fn swing(swing: Swing) -> Formation {
    Formation::with_velocity(Vec2::ZERO, move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            let (x, angle) = match swing {
                Swing::Left => (crate::WIDTH / 2. + 4., PI + PI / 6.),
                Swing::Right => (-crate::WIDTH / 2. - 4., -PI / 6.),
            };

            for i in 0..4 {
                let enemy = root
                    .spawn((
                        Swarm,
                        Platoon(root.target_entity()),
                        EmitterDelay::new(0.5 + 0.15 * i as f32),
                        Transform::from_xyz(x, 0., ENEMY_Z),
                        LinearVelocity::default(),
                    ))
                    .id();
                root.commands().entity(enemy).animation().insert(sequence((
                    tween(
                        Duration::from_secs_f32(i as f32 * 6. / SWARM_SPEED),
                        EaseKind::Linear,
                        enemy
                            .into_target()
                            .with(linear_velocity(Vec2::ZERO, Vec2::ZERO)),
                    ),
                    tween(
                        Duration::from_secs_f32(1. + i as f32 * 0.6),
                        EaseKind::Linear,
                        enemy.into_target().with(linear_velocity(
                            Vec2::from_angle(angle).normalize() * SWARM_SPEED,
                            Vec2::NEG_Y * SWARM_SPEED
                                + Vec2::new((3. - i as f32) * 2., (3. - i as f32) * 4.),
                        )),
                    ),
                )));
            }
        });
    })
}

//const SWARM_THRESHOLD: f32 = WIDTH * 1.25;
//
//#[derive(Debug, Component)]
//pub enum SwarmMovement {
//    Left,
//    Right,
//}
//
//pub(super) fn swarm_movement(
//    mut q: Query<(&mut LinearVelocity, &GlobalTransform, &SwarmMovement)>,
//) {
//    for (mut velocity, transform, movement) in &mut q {
//        let translation = transform.compute_transform().translation;
//
//        match movement {
//            SwarmMovement::Left => {
//                if translation.x > SWARM_THRESHOLD && velocity.x > 0. {
//                    velocity.x = -SWARM_SPEED;
//                } else if translation.x < -SWARM_THRESHOLD && velocity.x < 0. {
//                    velocity.x = SWARM_SPEED;
//                } else if velocity.x == 0. {
//                    velocity.x = SWARM_SPEED;
//                }
//            }
//            SwarmMovement::Right => {
//                if translation.x < SWARM_THRESHOLD && velocity.x < 0. {
//                    velocity.x = -SWARM_SPEED;
//                } else if translation.x > -SWARM_THRESHOLD && velocity.x > 0. {
//                    velocity.x = SWARM_SPEED;
//                } else if velocity.x == 0. {
//                    velocity.x = SWARM_SPEED;
//                }
//            }
//        }
//
//        if (-WIDTH * 0.5..WIDTH * 0.5).contains(&translation.x) {
//            let proportion = (translation.x + WIDTH * 0.5) / WIDTH;
//
//            let proportion = if velocity.x > 0. {
//                proportion
//            } else {
//                1. - proportion
//            };
//
//            let sin = ((proportion * core::f32::consts::PI).cos() - 0.5) * -10.;
//            velocity.y = sin;
//        } else {
//            velocity.y = 0.;
//        }
//    }
//}
