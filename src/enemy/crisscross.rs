use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use super::formation::animate_entrance_with;
use super::formation::powerup;
use super::movement::Figure8;
use super::timeline::LARGEST_SPRITE_SIZE;
use crate::bullet::BlueOrb;
use crate::bullet::RedOrb;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::EmitterSample;
use crate::bullet::emitter::ORB_SPEED;
use crate::bullet::emitter::PulseState;
use crate::bullet::emitter::PulseTimer;
use crate::bullet::homing::HomingRotate;
use crate::player::Player;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBehavior;
use crate::sprites::SpriteBundle;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

const CRISSCROSS_WAIT_RATE: f32 = 1.25;
const CRISSCROSS_SHOT_RATE: f32 = 0.15;
const CRISSCROSS_WAVES: usize = 5;

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
                position: Vec2::ZERO,
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
                (CrissCross, ChildOf(platoon), Platoon(platoon)),
                Figure8 {
                    radius: 15.,
                    speed: 0.5,
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
#[require(Transform, BulletModifiers, CrisscrossState)]
pub struct CrisscrossEmitter;

#[derive(Component, Default)]
pub enum CrisscrossState {
    #[default]
    Cross,
    Plus,
}

impl CrisscrossEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                Entity,
                &mut CrisscrossEmitter,
                Option<&mut PulseTimer>,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
                &mut CrisscrossState,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        for (entity, _emitter, timer, mods, parent, transform, mut state) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(PulseTimer::new(
                    mods.rate,
                    CRISSCROSS_WAIT_RATE,
                    CRISSCROSS_SHOT_RATE,
                    CRISSCROSS_WAVES,
                ));
                continue;
            };

            if !timer.just_finished(&time) {
                continue;
            }

            if matches!(timer.state, PulseState::Wait) {
                match *state {
                    CrisscrossState::Cross => *state = CrisscrossState::Plus,
                    CrisscrossState::Plus => *state = CrisscrossState::Cross,
                }
                continue;
            }

            let new_transform = transform.compute_transform();
            let angle_offset = match *state {
                CrisscrossState::Cross => std::f32::consts::PI / 4.,
                CrisscrossState::Plus => 0.,
            };
            let bullets = 4;
            for angle in 0..bullets {
                let angle = (angle as f32 / bullets as f32) * std::f32::consts::TAU
                    // + timer.current_pulse() as f32 * std::f32::consts::PI * 0.01
                    + angle_offset;

                commands.spawn((
                    RedOrb,
                    LinearVelocity(Vec2::from_angle(angle) * ORB_SPEED * mods.speed),
                    new_transform,
                ));
            }

            writer.write(EmitterSample(EmitterBullet::Orb));
        }
    }
}
