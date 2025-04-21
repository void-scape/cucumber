use super::{
    Bullet, BulletRate, BulletSpeed, BulletSprite, BulletTimer, BulletType, Polarity,
    homing::{Heading, Homing},
};
use crate::{auto_collider::ImageCollider, enemy::Enemy, health::Damage};
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use physics::{
    layers::{self, TriggersWith},
    prelude::Velocity,
};
use std::{f32::consts::PI, marker::PhantomData, time::Duration};

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                SoloEmitter::<layers::Enemy>::shoot_bullets,
                SoloEmitter::<layers::Player>::shoot_bullets,
                DualEmitter::<layers::Enemy>::shoot_bullets,
                DualEmitter::<layers::Player>::shoot_bullets,
                HomingEmitter::<layers::Enemy, Enemy>::shoot_bullets,
                HomingEmitter::<layers::Player, crate::player::Player>::shoot_bullets,
            ),
        );
    }
}

#[derive(Component, Default)]
#[require(
    BulletRate, BulletSpeed, BulletSprite(|| BulletSprite::from_cell(0, 0)), Polarity,
    Visibility(|| Visibility::Hidden)
)]
pub struct SoloEmitter<T>(PhantomData<fn() -> T>);

impl<T: Component> SoloEmitter<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> SoloEmitter<T> {
    fn shoot_bullets(
        mut emitters: Query<
            (Entity, Option<&mut BulletTimer>, &Polarity, &Parent),
            With<SoloEmitter<T>>,
        >,
        parents: Query<(&GlobalTransform, Option<&BulletRate>, Option<&BulletSpeed>)>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, timer, polarity, parent) in emitters.iter_mut() {
            let Ok((parent, rate, speed)) = parents.get(parent.get()) else {
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

            let mut new_transform = parent.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            timer.timer.set_duration(duration);

            commands.spawn((
                BulletType::Basic,
                Velocity(polarity.to_vec2() * 150.0 * speed.0),
                new_transform,
                TriggersWith::<T>::default(),
                Damage::new(1),
            ));

            commands
                .spawn((
                    SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                    PlaybackSettings {
                        volume: Volume::Decibels(-18.0),
                        ..PlaybackSettings::ONCE
                    },
                ))
                .effect(BandPassNode::new(1000.0, 4.0));
        }
    }
}

#[derive(Component, Default)]
#[require(
    BulletRate, BulletSpeed, BulletSprite(|| BulletSprite::from_cell(0, 0)),
    Polarity, Visibility(|| Visibility::Hidden)
)]
pub struct DualEmitter<T>(f32, PhantomData<fn() -> T>);

impl<T: Component> DualEmitter<T> {
    pub fn new(width: f32) -> Self {
        Self(width, PhantomData)
    }
}

impl<T: Component> DualEmitter<T> {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &DualEmitter<T>,
            Option<&mut BulletTimer>,
            &Polarity,
            &Parent,
        )>,
        parents: Query<(&GlobalTransform, Option<&BulletRate>, Option<&BulletSpeed>)>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, polarity, parent) in emitters.iter_mut() {
            let Ok((parent, rate, speed)) = parents.get(parent.get()) else {
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

            let mut new_transform = parent.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            timer.timer.set_duration(duration);

            commands.spawn((
                BulletType::Basic,
                Velocity(polarity.to_vec2() * 150.0 * speed.0),
                {
                    let mut t = new_transform;
                    t.translation.x -= emitter.0;
                    t
                },
                TriggersWith::<T>::default(),
                Damage::new(1),
            ));

            commands.spawn((
                BulletType::Basic,
                Velocity(polarity.to_vec2() * 150.0 * speed.0),
                {
                    new_transform.translation.x += emitter.0;
                    new_transform
                },
                TriggersWith::<T>::default(),
                Damage::new(1),
            ));

            commands
                .spawn((
                    SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                    PlaybackSettings {
                        volume: Volume::Decibels(-18.0),
                        ..PlaybackSettings::ONCE
                    },
                ))
                .effect(BandPassNode::new(1000.0, 4.0));
        }
    }
}

#[derive(Component, Default)]
#[require(BulletRate, BulletSpeed, Polarity)]
pub struct HomingEmitter<T, U>(PhantomData<fn() -> (T, U)>);

impl<T: Component, U: Component> HomingEmitter<T, U> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component, U: Component> HomingEmitter<T, U> {
    fn shoot_bullets(
        mut emitters: Query<
            (Entity, Option<&mut BulletTimer>, &Polarity, &Parent),
            With<HomingEmitter<T, U>>,
        >,
        parents: Query<(&GlobalTransform, Option<&BulletRate>)>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, timer, polarity, parent) in emitters.iter_mut() {
            let Ok((parent, rate)) = parents.get(parent.get()) else {
                continue;
            };
            let rate = rate.copied().unwrap_or_default();
            let duration = Duration::from_secs_f32(0.33 / rate.0);

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(BulletTimer {
                    timer: Timer::new(duration, TimerMode::Repeating),
                });
                continue;
            };

            let mut new_transform = parent.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            timer.timer.set_duration(duration);

            commands.spawn((
                BulletSprite::from_cell(5, 2),
                Bullet,
                ImageCollider,
                Velocity::default(),
                Homing::<U>::new(),
                Heading {
                    speed: 125.0,
                    direction: PI / 2.0,
                },
                new_transform,
                TriggersWith::<T>::default(),
                Damage::new(1),
            ));

            commands
                .spawn((
                    SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                    PlaybackSettings {
                        volume: Volume::Decibels(-18.0),
                        ..PlaybackSettings::ONCE
                    },
                ))
                .effect(BandPassNode::new(1000.0, 4.0));
        }
    }
}
