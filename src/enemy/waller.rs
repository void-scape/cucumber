use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use crate::bullet::emitter::TargetPlayer;
use crate::bullet::emitter::WallEmitter;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBundle;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;

#[derive(Default, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(20.),
    LowHealthEffects,
    WallEmitter,
    TargetPlayer,
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
