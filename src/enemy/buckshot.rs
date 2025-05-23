use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use super::timeline::LARGEST_SPRITE_SIZE;
use crate::bullet::BlueOrb;
use crate::bullet::emitter::BulletCommands;
use crate::bullet::emitter::BulletModifiers;
use crate::bullet::emitter::BulletSpeed;
use crate::bullet::emitter::Emitter;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterCtx;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::PulseTimer;
use crate::bullet::emitter::ShootEmitter;
use crate::bullet::emitter::Target;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBundle;
use crate::{effects::Explosion, health::Health, sprites::CellSprite};
use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

const SHOT: f32 = 0.2;
const WAIT: f32 = 2.;
const WAVES: usize = 2;
const BULLET_SPEED: f32 = 120.;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(6.),
    LowHealthEffects,
    BuckShotEmitter,
    Explosion::Big,
    FacePlayer,
)]
pub struct BuckShot;

impl BuckShot {
    fn sprites() -> SpriteBundle {
        SpriteBundle::new([
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(3, 1),
                z: 0.,
            }),
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(3, 2),
                z: 1.,
            }),
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(3, 3),
                z: -1.,
            }),
        ])
    }
}

pub fn left() -> Formation {
    Formation::with_velocity(
        Vec2::ZERO,
        |formation: &mut EntityCommands, server: &AssetServer| {
            formation.with_children(|root| {
                let platoon = root.target_entity();

                animate_entrance(
                    &server,
                    &mut root.commands(),
                    (
                        ChildOf(platoon),
                        EmitterDelay::new(1.5),
                        BuckShot,
                        Platoon(platoon),
                        Transform::from_xyz(0., 0., 0.),
                        ColliderDisabled,
                    ),
                    None,
                    1.5,
                    Vec3::new(0., 12., 0.),
                    Vec3::new(-25., -50., 0.),
                    Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                    Quat::default(),
                );
            });
        },
    )
}

pub fn right() -> Formation {
    Formation::with_velocity(
        Vec2::ZERO,
        |formation: &mut EntityCommands, server: &AssetServer| {
            formation.with_children(|root| {
                let platoon = root.target_entity();

                animate_entrance(
                    &server,
                    &mut root.commands(),
                    (
                        ChildOf(platoon),
                        EmitterDelay::new(1.5),
                        BuckShot,
                        Platoon(platoon),
                        Transform::from_xyz(0., 0., 0.),
                        ColliderDisabled,
                    ),
                    None,
                    1.5,
                    Vec3::new(0., 12., 0.),
                    Vec3::new(25., -50., 0.),
                    Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                    Quat::default(),
                );
            });
        },
    )
}

pub fn double() -> Formation {
    Formation::with_velocity(
        Vec2::ZERO,
        |formation: &mut EntityCommands, server: &AssetServer| {
            formation.with_children(|root| {
                let platoon = root.target_entity();

                animate_entrance(
                    &server,
                    &mut root.commands(),
                    (
                        ChildOf(platoon),
                        EmitterDelay::new(1.5),
                        BuckShot,
                        Platoon(platoon),
                        Transform::from_xyz(-30., 0., 0.),
                        ColliderDisabled,
                    ),
                    None,
                    1.5,
                    Vec3::new(-crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                    Vec3::new(-30., -40., 0.),
                    Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                    Quat::default(),
                );

                animate_entrance(
                    &server,
                    &mut root.commands(),
                    (
                        ChildOf(platoon),
                        EmitterDelay::new(1.5),
                        BuckShot,
                        Platoon(platoon),
                        Transform::from_xyz(-30., 0., 0.),
                        ColliderDisabled,
                    ),
                    Some(1.),
                    1.5,
                    Vec3::new(-crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                    Vec3::new(30., -40., 0.),
                    Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                    Quat::default(),
                );
            });
        },
    )
}

#[derive(Clone, Copy, Component)]
#[require(Transform, Emitter, BulletSpeed::new(BULLET_SPEED), Target::player())]
pub struct BuckShotEmitter {
    waves: usize,
    shot_dur: f32,
    wait_dur: f32,
}

impl Default for BuckShotEmitter {
    fn default() -> Self {
        Self::new(WAVES, WAIT, SHOT)
    }
}

impl BuckShotEmitter {
    pub fn new(waves: usize, wait_dur: f32, shot_dur: f32) -> Self {
        Self {
            waves,
            wait_dur,
            shot_dur,
        }
    }
}

impl ShootEmitter for BuckShotEmitter {
    type Timer = PulseTimer;

    fn timer(&self, mods: &BulletModifiers) -> Self::Timer {
        PulseTimer::ready(mods.rate, self.wait_dur, self.shot_dur, self.waves)
    }

    fn spawn_bullets(
        &self,
        mut commands: BulletCommands,
        transform: Transform,
        _ctx: EmitterCtx<Self::Timer>,
    ) {
        let angles = [-std::f32::consts::PI / 6., 0., std::f32::consts::PI / 6.];
        for angle in angles.into_iter() {
            commands.spawn_angled(angle, (BlueOrb, transform));
        }
    }

    fn sample() -> Option<EmitterBullet> {
        Some(EmitterBullet::Orb)
    }
}

pub fn track_player(mut emitters: Query<(&mut Target, &PulseTimer), With<BuckShotEmitter>>) {
    for (mut target, timer) in emitters.iter_mut() {
        target.enable(timer.is_waiting());
    }
}
