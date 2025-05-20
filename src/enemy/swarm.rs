use super::Enemy;
use super::FaceVelocity;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use super::timeline::ENEMY_Z;
use crate::bullet::Arrow;
use crate::bullet::BulletTimer;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::EmitterSample;
use crate::bullet::emitter::EmitterState;
use crate::player::Player;
use crate::{
    Layer,
    auto_collider::ImageCollider,
    bullet::emitter::{BulletModifiers, Rate},
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
const BULLET_RATE: f32 = 0.8;
const BULLET_SPEED: f32 = 90.;
const MAX_SHOTS: usize = 3;

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
    Shots,
)]
pub struct Swarm;

#[derive(Default, Component)]
pub struct Shots(usize);

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
#[require(
    Transform,
    EmitterState,
    BulletModifiers,
    BulletTimer::ready(BULLET_RATE)
)]
pub struct SwarmEmitter;

impl SwarmEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                &mut EmitterState,
                &mut BulletTimer,
                &mut Shots,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
            ),
            (Without<EmitterDelay>, With<SwarmEmitter>),
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        player: Single<&Transform, With<Player>>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (mut state, mut timer, mut shots, mods, child_of, transform) in emitters.iter_mut() {
            if !state.enabled {
                continue;
            }

            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }

            let new_transform = transform.compute_transform();
            let to_player =
                (player.translation.xy() - new_transform.translation.xy()).normalize_or(Vec2::ONE);
            commands.spawn((
                Arrow,
                LinearVelocity(to_player * BULLET_SPEED * mods.speed),
                new_transform.with_rotation(Quat::from_rotation_z(
                    to_player.to_angle() - PI / 2.0 + PI / 4.,
                )),
            ));
            shots.0 += 1;
            if shots.0 >= MAX_SHOTS {
                state.enabled = false;
            }

            writer.write(EmitterSample(EmitterBullet::Arrow));
        }
    }
}
