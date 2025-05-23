use std::f32::consts::PI;

use super::Enemy;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance_with;
use super::movement::Figure8;
use crate::bullet::BlueOrb;
use crate::bullet::RedOrb;
use crate::bullet::emitter::BulletCommands;
use crate::bullet::emitter::BulletSpeed;
use crate::bullet::emitter::Emitter;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterCtx;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::ORB_SPEED;
use crate::bullet::emitter::PulseTimer;
use crate::bullet::emitter::ShootEmitter;
use crate::bullet::emitter::Target;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBehavior;
use crate::sprites::SpriteBundle;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;

const WAIT: f32 = 1.25;
const SHOT: f32 = 0.15;
const WAVES: usize = 5;
const BULLET_SPEED: f32 = ORB_SPEED;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(15.),
    LowHealthEffects,
    CrisscrossEmitter,
    Explosion::Big,
)]
pub struct CrissCross;

impl CrissCross {
    fn sprites() -> SpriteBundle {
        SpriteBundle::new([
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(4, 1),
                z: 0.,
            }),
            MultiSprite::Dynamic {
                sprite: CellSprite {
                    path: "ships.png",
                    size: CellSize::TwentyFour,
                    cell: UVec2::new(4, 2),
                    z: -1.,
                },
                behavior: SpriteBehavior::Crisscross,
                transform: Transform::from_rotation(Quat::from_rotation_z(PI / 4.)),
            },
        ])
    }
}

pub fn single(position: Vec2) -> Formation {
    Formation::with_velocity(
        Vec2::ZERO,
        move |formation: &mut EntityCommands, server: &AssetServer| {
            let platoon = formation.id();
            animate_entrance_with(
                server,
                &mut formation.commands(),
                (
                    CrissCross,
                    ChildOf(platoon),
                    Platoon(platoon),
                    EmitterDelay::new(1.5),
                ),
                Figure8 {
                    radius: 15.,
                    speed: -0.5,
                },
                None,
                1.5,
                Vec3::new(0., 12., 0.),
                position.extend(0.),
                Quat::default(),
                Quat::default(),
            );
        },
    )
}

#[derive(Default, Component)]
#[require(Transform, Emitter, BulletSpeed::new(BULLET_SPEED))]
pub struct CrisscrossEmitter(pub CrisscrossState);

#[derive(Default)]
pub enum CrisscrossState {
    #[default]
    Cross,
    Plus,
}

impl ShootEmitter for CrisscrossEmitter {
    type Timer = PulseTimer;

    fn timer(&self, mods: &BulletModifiers) -> Self::Timer {
        PulseTimer::ready(mods.rate, WAIT, SHOT, WAVES)
    }

    fn spawn_bullets(
        &self,
        mut commands: BulletCommands,
        transform: Transform,
        _ctx: EmitterCtx<Self::Timer>,
    ) {
        let angle_offset = match self.0 {
            CrisscrossState::Cross => std::f32::consts::PI / 4.,
            CrisscrossState::Plus => 0.,
        };
        let bullets = 4;
        for angle in 0..bullets {
            let angle = (angle as f32 / bullets as f32) * std::f32::consts::TAU + angle_offset;
            let mut entity = commands.spawn_angled(angle, transform);
            match self.0 {
                CrisscrossState::Plus => entity.insert(RedOrb),
                CrisscrossState::Cross => entity.insert(BlueOrb),
            };
        }
    }

    fn sample() -> Option<EmitterBullet> {
        Some(EmitterBullet::Orb)
    }
}

pub fn swivel(mut emitters: Query<(&mut CrisscrossEmitter, &PulseTimer)>) {
    for (mut emitter, timer) in emitters.iter_mut() {
        if timer.just_finished() && timer.is_waiting() {
            emitter.0 = match emitter.0 {
                CrisscrossState::Cross => CrisscrossState::Plus,
                CrisscrossState::Plus => CrisscrossState::Cross,
            }
        }
    }
}
