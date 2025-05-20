use super::Enemy;
use super::FacePlayer;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use super::timeline::LARGEST_SPRITE_SIZE;
use crate::bullet::BlueOrb;
use crate::bullet::BulletTimer;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::EmitterSample;
use crate::bullet::emitter::ORB_SPEED;
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

const BULLET_RATE: f32 = 1.5;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(30.),
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
#[require(Transform, BulletModifiers, BulletTimer::ready(BULLET_RATE))]
pub struct WallEmitter {
    bullets: usize,
    dir: Vec2,
    gap: f32,
}

#[derive(Default, Component)]
pub struct TargetPlayer;

impl Default for WallEmitter {
    fn default() -> Self {
        Self::new(Vec2::NEG_Y, 5, 25.)
    }
}

impl WallEmitter {
    pub fn new(dir: Vec2, bullets: usize, gap: f32) -> Self {
        assert!(dir != Vec2::ZERO);
        Self {
            bullets,
            dir: dir.normalize(),
            gap,
        }
    }

    pub fn from_dir(dir: Vec2) -> Self {
        Self {
            dir,
            ..Default::default()
        }
    }
}

fn rotate_around(vec: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let translated_x = vec.x - center.x;
    let translated_y = vec.y - center.y;

    let rotated_x = translated_x * angle.cos() - translated_y * angle.sin();
    let rotated_y = translated_x * angle.sin() + translated_y * angle.cos();

    Vec2 {
        x: rotated_x + center.x,
        y: rotated_y + center.y,
    }
}

impl WallEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                &WallEmitter,
                &mut BulletTimer,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
                Option<&TargetPlayer>,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        player: Single<&Transform, With<Player>>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (emitter, mut timer, mods, parent, transform, target_player) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += new_transform.rotation.mul_vec3(Vec3::NEG_Y) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }

            let x_gap = emitter.gap;
            let center_x = (emitter.bullets - 1) as f32 * x_gap / 2.0;

            let dir = if target_player.is_none() {
                emitter.dir
            } else {
                (player.translation.xy() - new_transform.translation.xy()).normalize_or_zero()
            };

            let bowl_depth = 20.;
            for i in 0..emitter.bullets {
                let x = i as f32 * x_gap - center_x;
                let y = bowl_depth * (x / center_x).powi(2);

                let mut t = new_transform;
                let p = rotate_around(
                    t.translation.xy() + Vec2::new(x, y),
                    t.translation.xy(),
                    dir.to_angle() + std::f32::consts::PI / 2.,
                )
                .extend(0.);
                t.translation = p;

                commands.spawn((
                    BlueOrb,
                    t,
                    LinearVelocity(dir * ORB_SPEED * mods.speed * 1.5),
                ));

                //let angles = [-std::f32::consts::PI / 6., 0., std::f32::consts::PI / 6.];
                //for angle in angles.into_iter() {
                //    commands.spawn((
                //        BlueOrb,
                //        HomingRotate,
                //        LinearVelocity(
                //            Vec2::from_angle(angle)
                //                .rotate(*to_player)
                //                .normalize_or_zero()
                //                * BULLET_SPEED
                //                * mods.speed,
                //        ),
                //        new_transform,
                //    ));
                //}
            }

            writer.write(EmitterSample(EmitterBullet::Orb));
        }
    }
}
