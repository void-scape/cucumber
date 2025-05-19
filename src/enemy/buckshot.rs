use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use super::timeline::LARGEST_SPRITE_SIZE;
use crate::bullet::emitter::BuckShotEmitter;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBundle;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(6.),
    LowHealthEffects,
    BuckShotEmitter,
    BulletModifiers {
        speed: 1.,
        ..Default::default()
    },
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

pub fn double_buck_shot() -> Formation {
    Formation::new(|formation: &mut EntityCommands, server: &AssetServer| {
        formation.with_children(|root| {
            let platoon = root.target_entity();

            animate_entrance(
                &server,
                &mut root.commands(),
                (
                    ChildOf(platoon),
                    BuckShot,
                    Platoon(platoon),
                    Transform::from_xyz(-30., 0., 0.),
                ),
                None,
                1.5,
                Vec3::new(-crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                Vec3::new(-30., -10., 0.),
                Quat::from_rotation_z(PI / 4.) + Quat::from_rotation_x(PI / 4.),
                Quat::default(),
            );

            animate_entrance(
                &server,
                &mut root.commands(),
                (
                    ChildOf(platoon),
                    BuckShot,
                    Platoon(platoon),
                    Transform::from_xyz(-30., 0., 0.),
                ),
                Some(1.),
                1.5,
                Vec3::new(-crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                Vec3::new(30., -10., 0.),
                Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                Quat::default(),
            );
        });
    })
}
