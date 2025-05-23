use crate::bullet::{
    BulletTimer, Polarity, RedOrb,
    emitter::{BulletModifiers, EmitterDelay, ORB_SPEED},
};
use crate::float_tween;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tween::{
    combinator::{sequence, tween},
    prelude::*,
};
use std::{f32::consts::PI, time::Duration};

const ORB_RATE: f32 = 0.1;

#[derive(Default, Clone, Copy, Component)]
#[require(Transform, BulletModifiers, Polarity, Visibility::Hidden, SpiralOffset)]
pub struct GradiusSpiralEmitter;

float_tween!(
    Component,
    SpiralOffset,
    0.,
    spiral_offset,
    SpiralOffsetTween
);

impl GradiusSpiralEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                Entity,
                &mut GradiusSpiralEmitter,
                Option<&mut BulletTimer>,
                &SpiralOffset,
                &BulletModifiers,
                &Polarity,
                &ChildOf,
                &GlobalTransform,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        for (entity, _emitter, timer, offset, mods, polarity, parent, transform) in
            emitters.iter_mut()
        {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let Some(mut timer) = timer else {
                let mut timer = Timer::new(Duration::from_secs_f32(ORB_RATE), TimerMode::Repeating);
                timer.tick(time.delta());

                const ROTATION: f32 = PI / 7.;
                const CYCLE_TIME: f32 = 3.;
                commands
                    .entity(entity)
                    .insert(BulletTimer { timer })
                    .animation()
                    .repeat(Repeat::Infinitely)
                    .insert(sequence((
                        tween(
                            Duration::from_secs_f32(CYCLE_TIME / 2.),
                            EaseKind::Linear,
                            entity.into_target().with(spiral_offset(0., ROTATION)),
                        ),
                        tween(
                            Duration::from_secs_f32(CYCLE_TIME / 2.),
                            EaseKind::Linear,
                            entity.into_target().with(spiral_offset(ROTATION, 0.)),
                        ),
                        tween(
                            Duration::from_secs_f32(CYCLE_TIME / 2.),
                            EaseKind::Linear,
                            entity.into_target().with(spiral_offset(0., ROTATION * 1.5)),
                        ),
                        tween(
                            Duration::from_secs_f32(CYCLE_TIME / 2.),
                            EaseKind::Linear,
                            entity.into_target().with(spiral_offset(ROTATION * 1.5, 0.)),
                        ),
                    )));

                continue;
            };

            timer.timer.tick(time.delta());
            if !timer.timer.just_finished() {
                continue;
            }

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            let bullets = 10;
            for angle in 0..bullets {
                let angle = (angle as f32 / bullets as f32) * 2. * std::f32::consts::PI + offset.0;
                commands.spawn((
                    RedOrb,
                    LinearVelocity(Vec2::from_angle(angle) * ORB_SPEED * mods.speed),
                    new_transform,
                ));
            }
        }
    }
}
