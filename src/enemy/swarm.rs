use super::Enemy;
use super::FaceVelocity;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use super::timeline::ENEMY_Z;
use crate::bullet::Arrow;
use crate::bullet::BulletTimer;
use crate::bullet::emitter::BulletCommands;
use crate::bullet::emitter::BulletSpeed;
use crate::bullet::emitter::Emitter;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterCtx;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::PulseLimit;
use crate::bullet::emitter::RotateBullet;
use crate::bullet::emitter::ShootEmitter;
use crate::bullet::emitter::ShotLimit;
use crate::bullet::emitter::Target;
use crate::{
    Layer, auto_collider::ImageCollider, bullet::emitter::BulletModifiers, effects::Explosion,
    health::Health, sprites::CellSprite,
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
const BULLET_RATE: f32 = 2.;
const BULLET_SPEED: f32 = 90.;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(1.),
    CellSprite::new8("ships.png", UVec2::new(3, 0)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    SwarmEmitter,
    Trauma::NONE,
    Explosion::Small,
    FaceVelocity,
    ShotLimit(3),
)]
pub struct Swarm;

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
                        ShotLimit(1),
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
                        EmitterDelay::new(0.5 + 0.25 * i as f32),
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

#[derive(Default, Component)]
#[require(Transform, Emitter, BulletSpeed::new(BULLET_SPEED), Target::player())]
pub struct SwarmEmitter;

impl ShootEmitter for SwarmEmitter {
    type Timer = BulletTimer;

    fn timer(&self, _: &BulletModifiers) -> Self::Timer {
        BulletTimer::ready(BULLET_RATE)
    }

    fn spawn_bullets(
        &self,
        mut commands: BulletCommands,
        transform: Transform,
        ctx: EmitterCtx<Self::Timer>,
    ) {
        commands
            .spawn((Arrow, transform))
            .look_at_offset(ctx.target, -PI / 2.0 + PI / 4.);
    }

    fn sample() -> Option<EmitterBullet> {
        Some(EmitterBullet::Arrow)
    }
}
