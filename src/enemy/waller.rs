use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use super::timeline::LARGEST_SPRITE_SIZE;
use crate::bullet::BlueOrb;
use crate::bullet::BulletTimer;
use crate::bullet::emitter::BulletCommands;
use crate::bullet::emitter::BulletSpeed;
use crate::bullet::emitter::Emitter;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterCtx;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::ORB_SPEED;
use crate::bullet::emitter::ShootEmitter;
use crate::bullet::emitter::Target;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBundle;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

const BULLET_RATE: f32 = 1.5;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(30.),
    LowHealthEffects,
    WallEmitter,
    BulletModifiers {
        speed: 0.8,
        ..Default::default()
    },
    Explosion::Big,
    FacePlayer,
)]
pub struct Waller;

impl Waller {
    fn sprites() -> SpriteBundle {
        SpriteBundle::new([
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(1, 1),
                z: 0.,
            }),
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(1, 2),
                z: -1.,
            }),
        ])
    }
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
                        Waller,
                        Platoon(platoon),
                        Transform::from_xyz(-30., 0., 0.),
                        ColliderDisabled,
                    ),
                    Some(1.),
                    1.5,
                    Vec3::new(crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
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
                        Waller,
                        Platoon(platoon),
                        Transform::from_xyz(-30., 0., 0.),
                        ColliderDisabled,
                    ),
                    None,
                    1.5,
                    Vec3::new(crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                    Vec3::new(30., -40., 0.),
                    Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                    Quat::default(),
                );
            });
        },
    )
}

#[derive(Component)]
#[require(Transform, Emitter, BulletSpeed::new(ORB_SPEED), Target::player())]
pub struct WallEmitter {
    pub bullets: usize,
    pub gap: f32,
    pub bowl: f32,
}

impl Default for WallEmitter {
    fn default() -> Self {
        Self {
            bullets: 4,
            gap: 18.,
            bowl: 10.,
        }
    }
}

impl ShootEmitter for WallEmitter {
    type Timer = BulletTimer;

    fn timer(&self, _mods: &BulletModifiers) -> Self::Timer {
        BulletTimer::ready(BULLET_RATE)
    }

    fn spawn_bullets(
        &self,
        mut commands: BulletCommands,
        transform: Transform,
        ctx: EmitterCtx<Self::Timer>,
    ) {
        let x_gap = self.gap;
        let center_x = (self.bullets - 1) as f32 * x_gap / 2.;

        for i in 0..self.bullets {
            let x = i as f32 * x_gap - center_x;
            let y = self.bowl * (x / center_x).powi(2);

            let mut transform = transform;
            transform.translation += (Vec2::from_angle(PI / 2.)
                .rotate(ctx.target.as_vec2())
                .rotate(Vec2::new(x, y)))
            .extend(0.);
            commands.spawn((BlueOrb, transform));
        }
    }

    fn sample() -> Option<EmitterBullet> {
        Some(EmitterBullet::Orb)
    }
}
