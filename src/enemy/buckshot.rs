use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use super::timeline::LARGEST_SPRITE_SIZE;
use crate::bullet::BlueOrb;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::EmitterSample;
use crate::bullet::emitter::PulseTimer;
use crate::bullet::homing::HomingRotate;
use crate::player::Player;
use crate::sprites::CellSize;
use crate::sprites::MultiSprite;
use crate::sprites::SpriteBundle;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use std::f32::consts::PI;

const BUCKSHOT_WAIT_RATE: f32 = 2.;
const BUCKSHOT_SHOT_RATE: f32 = 0.2;
const BUCKSHOT_WAVES: usize = 2;
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

pub fn double_buck_shot() -> Formation {
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
#[require(Transform, BulletModifiers)]
pub struct BuckShotEmitter {
    waves: usize,
    shot_dur: f32,
    wait_dur: f32,
}

impl Default for BuckShotEmitter {
    fn default() -> Self {
        Self {
            waves: BUCKSHOT_WAVES,
            shot_dur: BUCKSHOT_SHOT_RATE,
            wait_dur: BUCKSHOT_WAIT_RATE,
        }
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

impl BuckShotEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                Entity,
                &mut BuckShotEmitter,
                Option<&mut PulseTimer>,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        player: Single<&Transform, With<Player>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
        mut to_player: Local<Vec2>,
    ) {
        for (entity, emitter, timer, mods, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let new_transform = transform.compute_transform().with_rotation(Quat::default());
            //new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            let Some(mut timer) = timer else {
                let mut timer =
                    PulseTimer::new(mods.rate, emitter.wait_dur, emitter.shot_dur, emitter.waves);
                timer.reset_active();
                commands.entity(entity).insert(timer);
                *to_player = player.translation.xy() - new_transform.translation.xy();
                continue;
            };

            if !timer.just_finished(&time) {
                if timer.is_waiting() {
                    *to_player = player.translation.xy() - new_transform.translation.xy();
                }
                continue;
            }

            let angles = [-std::f32::consts::PI / 6., 0., std::f32::consts::PI / 6.];
            for angle in angles.into_iter() {
                commands.spawn((
                    BlueOrb,
                    HomingRotate,
                    LinearVelocity(
                        Vec2::from_angle(angle)
                            .rotate(*to_player)
                            .normalize_or_zero()
                            * BULLET_SPEED
                            * mods.speed,
                    ),
                    new_transform,
                ));
            }

            writer.write(EmitterSample(EmitterBullet::Orb));
        }
    }
}
