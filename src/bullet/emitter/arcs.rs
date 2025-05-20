use super::{
    BULLET_DAMAGE, BULLET_SPEED, BasicBullet, BulletModifiers, BulletTimer, EmitterBullet,
    EmitterSample, EmitterState, PLAYER_BULLET_RATE,
};
use crate::{
    bullet::{Mine, homing::HomingRotate},
    health::{Damage, Dead, Health},
    player::Player,
    tween::{DespawnTweenFinish, OnEnd},
};
use avian2d::prelude::LinearVelocity;
use bevy::{math::NormedVectorSpace, prelude::*};
use bevy_tween::{
    combinator::{sequence, tween},
    interpolate::translation_to,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::time::Duration;

// NOTE: we assume this is always on an enemy
#[derive(Component, Default)]
#[require(Transform, BulletModifiers, EmitterState, BulletTimer { timer: Timer::new(Duration::from_secs(3), TimerMode::Repeating) })]
pub struct ArcsEmitter;

impl ArcsEmitter {
    pub(super) fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &EmitterState,
            &Self,
            &mut BulletTimer,
            &BulletModifiers,
            &GlobalTransform,
            &ChildOf,
        )>,
        parents: Query<Option<&BulletModifiers>, With<Children>>,
        player: Query<&GlobalTransform, With<Player>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        let delta = time.delta();
        let player_position = player
            .single()
            .map(|p| p.compute_transform().translation)
            .unwrap_or_default();

        for (_entity, state, emitter, mut timer, mods, transform, child_of) in emitters.iter_mut() {
            if !state.enabled {
                continue;
            }

            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation -= Vec3::Y * 4.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            let duration = mods.rate.duration(PLAYER_BULLET_RATE);
            timer.timer.set_duration(duration);

            let direction = (player_position - new_transform.translation).normalize();
            // let angle = direction.xy().to_angle();
            // new_transform.rotate_z(angle - std::f32::consts::FRAC_PI_2);

            let intended_speed = BULLET_SPEED * mods.speed * 0.66;
            let distance = player_position.distance(new_transform.translation);
            let duration = Duration::from_secs_f32(distance / intended_speed);

            let mut bullet = commands.spawn((
                Mine,
                // LinearVelocity(direction.xy() * BULLET_SPEED * mods.speed),
                new_transform,
                Damage::new(BULLET_DAMAGE * mods.damage),
                HomingRotate,
            ));

            let position = bullet.id().into_target();
            let mut position = position.state(new_transform.translation);

            bullet.animation().insert(sequence((tween(
                duration,
                EaseKind::QuadraticOut,
                position.with(translation_to(player_position)),
            ),)));

            let id = bullet.id();
            let on_end = OnEnd::new(&mut commands, move |mut commands: Commands| {
                commands
                    .entity(id)
                    .entry::<Health>()
                    .and_modify(|mut h| h.damage_all());
            });
            commands.entity(id).insert(on_end);

            writer.write(EmitterSample(EmitterBullet::Bullet));
        }
    }
}
