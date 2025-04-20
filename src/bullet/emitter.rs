use super::{BulletRate, BulletSpeed, BulletSprite, BulletTimer, BulletType, Polarity};
use crate::health::Damage;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use physics::layers::{self, TriggersWith};
use std::{marker::PhantomData, time::Duration};

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                DualEmitter::<layers::Enemy>::shoot_bullets,
                DualEmitter::<layers::Player>::shoot_bullets,
            ),
        );
    }
}

#[derive(Component, Default)]
#[require(BulletRate, BulletSpeed, BulletSprite(|| BulletSprite::from_cell(0, 0)), Polarity)]
pub struct SoloEmitter;

#[derive(Component, Default)]
#[require(BulletRate, BulletSpeed, BulletSprite(|| BulletSprite::from_cell(0, 0)), Polarity)]
pub struct DualEmitter<T>(PhantomData<fn() -> T>);

impl<T: Component> DualEmitter<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> DualEmitter<T> {
    fn shoot_bullets(
        mut emitters: Query<
            (
                Entity,
                Option<&mut BulletTimer>,
                &BulletRate,
                &BulletSpeed,
                &BulletSprite,
                &Transform,
                &Polarity,
            ),
            With<DualEmitter<T>>,
        >,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, timer, rate, speed, sprite, transform, polarity) in emitters.iter_mut() {
            let mut new_transform = transform.clone();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 5.0;

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(BulletTimer {
                    timer: Timer::new(Duration::from_secs_f32(0.25 * rate.0), TimerMode::Repeating),
                });
                continue;
            };

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }

            commands.spawn((
                BulletType::Basic,
                Polarity::North,
                {
                    let mut t = new_transform.clone();
                    t.translation.x -= 3.;
                    t
                },
                TriggersWith::<T>::default(),
                Damage::new(1),
            ));

            commands.spawn((
                BulletType::Basic,
                Polarity::North,
                {
                    new_transform.translation.x += 3.;
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
