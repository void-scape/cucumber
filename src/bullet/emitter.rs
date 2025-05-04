use super::{
    Bullet, BulletRate, BulletSpeed, BulletSprite, BulletTimer, BulletType, Polarity,
    homing::{Heading, Homing, HomingRotate, TurnSpeed},
};
use crate::{Layer, auto_collider::ImageCollider, enemy::EnemyType, health::Damage};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use std::{f32::consts::PI, marker::PhantomData, time::Duration};

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                SoloEmitter::shoot_bullets,
                DualEmitter::shoot_bullets,
                HomingEmitter::<EnemyType>::shoot_bullets,
                HomingEmitter::<crate::player::Player>::shoot_bullets,
            ),
        );
    }
}

/// Determines the base speed of fired bullets. This value is multiplied by the parent's
/// [`BulletSpeed`].
#[derive(Component)]
pub struct BaseSpeed(pub f32);

#[derive(Component)]
#[require(
    Transform,
    BaseSpeed(150.),
    BulletSprite::from_cell(0, 0),
    Polarity,
    Visibility::Hidden
)]
pub struct SoloEmitter(Layer);

impl SoloEmitter {
    pub fn player() -> Self {
        Self(Layer::Player)
    }

    pub fn enemy() -> Self {
        Self(Layer::Enemy)
    }
}

const BULLET_RANGE: core::ops::Range<f64> = 0.9..1.1;

impl SoloEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &SoloEmitter,
            Option<&mut BulletTimer>,
            &BaseSpeed,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<(Option<&BulletRate>, Option<&BulletSpeed>)>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, base_speed, polarity, child_of, transform) in
            emitters.iter_mut()
        {
            let Ok((rate, speed)) = parents.get(child_of.parent()) else {
                continue;
            };
            let rate = rate.copied().unwrap_or_default();
            let speed = speed.copied().unwrap_or_default();

            let duration = Duration::from_secs_f32(0.25 / rate.0);
            let Some(mut timer) = timer else {
                commands.entity(entity).insert(BulletTimer {
                    timer: Timer::new(duration, TimerMode::Repeating),
                });
                continue;
            };

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            timer.timer.set_duration(duration);

            commands.spawn((
                BulletType::Basic,
                LinearVelocity(polarity.to_vec2() * base_speed.0 * speed.0),
                new_transform,
                Bullet::target_layer(emitter.0),
                Damage::new(1.0),
            ));

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                PitchRange(BULLET_RANGE),
                PlaybackSettings {
                    volume: Volume::Decibels(-12.0),
                    ..PlaybackSettings::ONCE
                },
                sample_effects![BandPassNode::new(1000.0, 4.0)],
            ));
        }
    }
}

#[derive(Component, Default)]
#[require(
    Transform,
    BaseSpeed(150.),
    BulletSprite::from_cell(0, 0),
    Polarity,
    Visibility::Hidden
)]
pub struct DualEmitter {
    target: Layer,
    width: f32,
}

impl DualEmitter {
    pub fn player(width: f32) -> Self {
        Self {
            target: Layer::Player,
            width,
        }
    }

    pub fn enemy(width: f32) -> Self {
        Self {
            target: Layer::Enemy,
            width,
        }
    }
}

impl DualEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &DualEmitter,
            Option<&mut BulletTimer>,
            &BaseSpeed,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<(Option<&BulletRate>, Option<&BulletSpeed>)>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, base_speed, polarity, parent, transform) in emitters.iter_mut()
        {
            let Ok((rate, speed)) = parents.get(parent.parent()) else {
                continue;
            };
            let rate = rate.copied().unwrap_or_default();
            let speed = speed.copied().unwrap_or_default();

            let duration = Duration::from_secs_f32(0.25 / rate.0);

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(BulletTimer {
                    timer: Timer::new(duration, TimerMode::Repeating),
                });
                continue;
            };

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            timer.timer.set_duration(duration);

            commands.spawn((
                BulletType::Basic,
                LinearVelocity(polarity.to_vec2() * base_speed.0 * speed.0),
                {
                    let mut t = new_transform;
                    t.translation.x -= emitter.width;
                    t
                },
                Bullet::target_layer(emitter.target),
                Damage::new(1.0),
            ));

            commands.spawn((
                BulletType::Basic,
                LinearVelocity(polarity.to_vec2() * base_speed.0 * speed.0),
                {
                    new_transform.translation.x += emitter.width;
                    new_transform
                },
                Bullet::target_layer(emitter.target),
                Damage::new(1.0),
            ));

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                PitchRange(BULLET_RANGE),
                PlaybackSettings {
                    volume: Volume::Decibels(-12.0),
                    ..PlaybackSettings::ONCE
                },
                sample_effects![BandPassNode::new(1000.0, 4.0)],
            ));
        }
    }
}

#[derive(Component, Default)]
#[require(Transform, BaseSpeed(125.), TurnSpeed, Polarity)]
pub struct HomingEmitter<T> {
    target: Layer,
    _filter: PhantomData<fn() -> T>,
}

impl<T: Component> HomingEmitter<T> {
    pub fn player() -> Self {
        Self {
            target: Layer::Player,
            _filter: PhantomData,
        }
    }

    pub fn enemy() -> Self {
        Self {
            target: Layer::Enemy,
            _filter: PhantomData,
        }
    }
}

impl<T: Component> HomingEmitter<T> {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &HomingEmitter<T>,
            Option<&mut BulletTimer>,
            &BaseSpeed,
            &TurnSpeed,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<(Option<&BulletRate>, Option<&BulletSpeed>)>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, base_speed, turn_speed, polarity, child_of, transform) in
            emitters.iter_mut()
        {
            let Ok((rate, speed)) = parents.get(child_of.parent()) else {
                continue;
            };
            let rate = rate.copied().unwrap_or_default();
            let speed = speed.copied().unwrap_or_default();

            let duration = Duration::from_secs_f32(0.33 / rate.0);

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(BulletTimer {
                    timer: Timer::new(duration, TimerMode::Repeating),
                });
                continue;
            };

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            timer.timer.set_duration(duration);

            let direction = match polarity {
                Polarity::North => PI / 2.0,
                Polarity::South => -PI / 2.0,
            };
            commands.spawn((
                BulletSprite::from_cell(5, 2),
                Bullet,
                ImageCollider,
                LinearVelocity::default(),
                Homing::<T>::new(),
                HomingRotate,
                *turn_speed,
                Heading {
                    speed: base_speed.0 * speed.0,
                    direction,
                },
                new_transform,
                Bullet::target_layer(emitter.target),
                Damage::new(1.0),
            ));

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                PitchRange(BULLET_RANGE),
                PlaybackSettings {
                    volume: Volume::Decibels(-12.0),
                    ..PlaybackSettings::ONCE
                },
                sample_effects![BandPassNode::new(1000.0, 4.0)],
            ));
        }
    }
}
