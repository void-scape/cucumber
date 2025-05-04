use super::{
    Bullet, BulletCollisionEvent, BulletRate, BulletSource, BulletSpeed, BulletSprite, BulletTimer,
    BulletType, Polarity,
    homing::{Heading, Homing, HomingRotate, TurnSpeed},
};
use crate::{
    HEIGHT, Layer,
    auto_collider::ImageCollider,
    enemy::EnemyType,
    health::{Damage, Health},
};
use avian2d::prelude::*;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
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
                LaserEmitter::laser,
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
        time: Res<Time<Physics>>,
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
        time: Res<Time<Physics>>,
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
        time: Res<Time<Physics>>,
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

#[derive(Component)]
#[require(Transform, BaseSpeed(30.), Polarity)]
#[component(on_insert = Self::on_insert_hook)]
pub struct LaserEmitter(Layer);

impl LaserEmitter {
    pub fn player() -> Self {
        Self(Layer::Player)
    }

    pub fn enemy() -> Self {
        Self(Layer::Enemy)
    }
}

impl LaserEmitter {
    fn on_insert_hook(mut world: DeferredWorld, context: HookContext) {
        // world.commands().run_system(||)

        let server = world.resource();
        let sprite = BulletSprite::from_cell(1, 8);
        let sprite = super::assets::sprite_rect8(server, sprite.path, sprite.cell);

        world.commands().entity(context.entity).with_children(|c| {
            let rot = Quat::from_rotation_z(core::f32::consts::FRAC_PI_2);

            c.spawn((
                sprite,
                Transform::default()
                    .with_translation(Vec3::new(0.0, 4.0, 10.0))
                    .with_rotation(rot),
            ));
        });
    }

    fn laser(
        spatial_query: SpatialQuery,
        mut emitters: Query<(
            Entity,
            &LaserEmitter,
            Option<&mut BulletTimer>,
            &BaseSpeed,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
            &Children,
        )>,
        mut child: Query<&mut Transform>,
        parents: Query<(Option<&BulletRate>, Option<&BulletSpeed>)>,
        time: Res<Time<Physics>>,
        mut targets: Query<(&mut Health, &GlobalTransform)>,
        mut writer: EventWriter<BulletCollisionEvent>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) -> Result {
        let delta = time.delta();

        for (entity, emitter, timer, base_speed, polarity, child_of, transform, children) in
            emitters.iter_mut()
        {
            let Ok((rate, speed)) = parents.get(child_of.parent()) else {
                continue;
            };
            let rate = rate.copied().unwrap_or_default();
            let speed = speed.copied().unwrap_or_default();

            let duration = Duration::from_secs_f32(0.25);
            if timer.is_none() {
                commands.entity(entity).insert(BulletTimer {
                    timer: Timer::new(duration, TimerMode::Repeating),
                });
            };

            let direction = polarity.to_vec2();
            let mut new_transform = transform.compute_transform();
            new_transform.translation += direction.extend(0.0) * 10.0;

            let child_entity = children
                .iter()
                .next()
                .ok_or("laser emitter should have child")?;

            let filter =
                SpatialQueryFilter::default().with_mask([emitter.0, Layer::Bounds, Layer::Debris]);

            if let Some(hit_data) = spatial_query.cast_ray(
                new_transform.translation.xy(),
                Dir2::from_xy(direction.x, direction.y).unwrap_or(Dir2::NORTH),
                HEIGHT,
                false,
                &filter,
            ) {
                let mut child = child.get_mut(child_entity)?;

                let target_scale = (hit_data.distance + 8.0) / 8.0;
                let difference = target_scale - child.scale.x;

                if difference < 0.0 {
                    child.scale.x = target_scale;
                } else {
                    child.scale.x += difference.max(base_speed.0) * delta.as_secs_f32() * speed.0;
                }

                child.translation.y = direction.y * (4.0 + child.scale.x * 8.0 / 2.0);

                if let Ok((mut target, target_transform)) = targets.get_mut(hit_data.entity) {
                    if (child.scale.x * 8.0 - hit_data.distance).abs() <= 16.0 {
                        target.damage(15.0 * delta.as_secs_f32());
                    }

                    if let Some(mut timer) = timer {
                        if timer.timer.tick(delta).just_finished() {
                            writer.write(BulletCollisionEvent::new(
                                UVec2::new(1, 8),
                                target_transform.compute_transform(),
                                match emitter.0 {
                                    Layer::Player => BulletSource::Enemy,
                                    Layer::Enemy => BulletSource::Player,
                                    _ => BulletSource::Enemy,
                                },
                            ));
                        }
                    }
                }
            }

            // if !timer.timer.tick(delta).just_finished() {
            //     continue;
            // }
            // timer.timer.set_duration(duration);

            // commands.spawn((
            //     BulletType::Basic,
            //     LinearVelocity(polarity.to_vec2() * base_speed.0 * speed.0),
            //     new_transform,
            //     Bullet::target_layer(emitter.0),
            //     Damage::new(1.0),
            // ));

            // commands.spawn((
            //     SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
            //     PitchRange(BULLET_RANGE),
            //     PlaybackSettings {
            //         volume: Volume::Decibels(-12.0),
            //         ..PlaybackSettings::ONCE
            //     },
            //     sample_effects![BandPassNode::new(1000.0, 4.0)],
            // ));
        }

        Ok(())
    }
}
