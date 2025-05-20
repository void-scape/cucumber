use super::Enemy;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use crate::bullet::BulletTimer;
use crate::bullet::Mine;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::EmitterSample;
use crate::player::Player;
use crate::{
    bullet::emitter::BulletModifiers, effects::Explosion, health::Health, sprites::CellSprite,
};
use avian2d::prelude::*;
use bevy::prelude::*;

const BULLET_RATE: f32 = 2.;
const BULLET_SPEED: f32 = 50.;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    CellSprite::new24("ships.png", UVec2::new(2, 1)),
    Health::full(25.),
    LowHealthEffects,
    MineEmitter,
    Explosion::Big
)]
pub struct MineThrower;

pub fn quad_mine_thrower() -> Formation {
    Formation::new(|formation: &mut EntityCommands, server: &AssetServer| {
        formation.with_children(|root| {
            let platoon = root.target_entity();

            animate_entrance(
                server,
                &mut root.commands(),
                (
                    MineThrower,
                    ChildOf(platoon),
                    Platoon(platoon),
                    Transform::from_xyz(-20., 0., 0.),
                ),
                None,
                1.5,
                Vec3::ZERO,
                Vec3::new(-20., 0., 0.),
                Quat::default(),
                Quat::default(),
            );
            animate_entrance(
                server,
                &mut root.commands(),
                (
                    MineThrower,
                    ChildOf(platoon),
                    Platoon(platoon),
                    Transform::from_xyz(-20., 0., 0.),
                ),
                None,
                1.5,
                Vec3::ZERO,
                Vec3::new(20., 0., 0.),
                Quat::default(),
                Quat::default(),
            );
            animate_entrance(
                server,
                &mut root.commands(),
                (
                    MineThrower,
                    ChildOf(platoon),
                    Platoon(platoon),
                    Transform::from_xyz(-20., 0., 0.),
                ),
                Some(1.),
                1.5,
                Vec3::ZERO,
                Vec3::new(40., 10., 0.),
                Quat::default(),
                Quat::default(),
            );
            animate_entrance(
                server,
                &mut root.commands(),
                (
                    MineThrower,
                    ChildOf(platoon),
                    Platoon(platoon),
                    Transform::from_xyz(-20., 0., 0.),
                ),
                Some(1.),
                1.5,
                Vec3::ZERO,
                Vec3::new(-40., 10., 0.),
                Quat::default(),
                Quat::default(),
            );
        });
    })
}

#[derive(Component, Default)]
#[require(Transform, BulletModifiers, BulletTimer::ready(BULLET_RATE))]
pub struct MineEmitter;

impl MineEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                &MineEmitter,
                &mut BulletTimer,
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
    ) {
        let delta = time.delta();

        for (_emitter, mut timer, mods, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += Vec3::NEG_Y * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }

            let to_player =
                (player.translation.xy() - new_transform.translation.xy()).normalize_or(Vec2::ONE);
            let velocity = Vec2::NEG_Y.with_x(0.5)
                * to_player.xy().with_y(-to_player.y)
                * BULLET_SPEED
                * mods.speed;
            commands.spawn((
                Mine,
                Rotation::radians(0.4),
                LinearVelocity(velocity),
                new_transform,
            ));

            writer.write(EmitterSample(EmitterBullet::Mine));
        }
    }
}
