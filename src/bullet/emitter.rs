use super::{
    BasicBullet, Bullet, BulletCollisionEvent, BulletSource, BulletSprite, BulletTimer, Lifetime,
    MaxLifetime, Mine, Missile, Orb, Polarity,
    homing::{Heading, Homing, HomingRotate, TurnSpeed},
};
use crate::{
    Avian, HEIGHT, Layer,
    enemy::Enemy,
    health::{Damage, Health},
    player::Player,
};
use avian2d::prelude::*;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_seedling::prelude::*;
use std::{f32::consts::PI, marker::PhantomData, time::Duration};

pub const BULLET_SPEED: f32 = 75.;
pub const MISSILE_SPEED: f32 = 65.;
pub const LASER_SPEED: f32 = 15.;
pub const MINE_SPEED: f32 = 50.;
pub const ORB_SPEED: f32 = 50.;

pub const BULLET_DAMAGE: f32 = 1.;
pub const MISSILE_DAMAGE: f32 = 1.;
pub const MINE_DAMAGE: f32 = 1.;
pub const ORB_DAMAGE: f32 = 1.;

const BULLET_RATE: f32 = 0.5;
const MISSILE_RATE: f32 = 0.5;
const MINE_RATE: f32 = 1.5;

const ORB_WAIT_RATE: f32 = 2.;
const ORB_SHOT_RATE: f32 = 0.4;
const ORB_WAVES: usize = 8;

pub const MISSILE_HEALTH: f32 = 3.;
pub const MINE_HEALTH: f32 = 5.;

const BULLET_PITCH_RANGE: core::ops::Range<f64> = 0.9..1.1;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct EmitterSet;

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Avian,
            (
                SoloEmitter::shoot_bullets,
                DualEmitter::shoot_bullets,
                LaserEmitter::laser,
                HomingEmitter::<Enemy>::shoot_bullets,
                HomingEmitter::<Player>::shoot_bullets,
                MineEmitter::shoot_bullets,
                OrbEmitter::shoot_bullets,
            )
                .in_set(EmitterSet),
        );
    }
}

/// Multipliers applied to the properties of bullet emitters.
///
/// Emitters will apply parent modifiers on top of their own with [`BulletModifiers::join`].
///
/// The base values for speed, damage, and rate are `[BULLET/MISSILE/LASER]_SPEED`, `1.0`, and
/// `[BULLET/MISSILE]_RATE` (laser has no rate). Rate is calculated as `[BULLET/MISSILE]_RATE / rate`.
#[derive(Clone, Copy, Component)]
pub struct BulletModifiers {
    pub speed: f32,
    pub damage: f32,
    pub rate: f32,
}

impl Default for BulletModifiers {
    fn default() -> Self {
        Self {
            speed: 1.,
            damage: 1.,
            rate: 1.,
        }
    }
}

impl BulletModifiers {
    pub fn join(&self, other: &Self) -> Self {
        Self {
            speed: self.speed * other.speed,
            damage: self.damage * other.damage,
            rate: self.rate * other.rate,
        }
    }
}

#[derive(Component)]
#[require(Transform, BulletModifiers, Polarity, Visibility::Hidden)]
pub struct SoloEmitter(Layer);

impl SoloEmitter {
    pub fn player() -> Self {
        Self(Layer::Player)
    }

    pub fn enemy() -> Self {
        Self(Layer::Enemy)
    }
}

impl SoloEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &SoloEmitter,
            Option<&mut BulletTimer>,
            &BulletModifiers,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, mods, polarity, child_of, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let duration = Duration::from_secs_f32(BULLET_RATE * 1. / mods.rate);
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
                BasicBullet,
                LinearVelocity(polarity.to_vec2() * BULLET_SPEED * mods.speed),
                new_transform,
                Bullet::target_layer(emitter.0),
                Damage::new(BULLET_DAMAGE * mods.damage),
            ));

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                PitchRange(BULLET_PITCH_RANGE),
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
#[require(Transform, BulletModifiers, Polarity, Visibility::Hidden)]
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
            &BulletModifiers,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, mods, polarity, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let duration = Duration::from_secs_f32(BULLET_RATE * 1. / mods.rate);
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
                BasicBullet,
                LinearVelocity(polarity.to_vec2() * BULLET_SPEED * mods.speed),
                {
                    let mut t = new_transform;
                    t.translation.x -= emitter.width;
                    t
                },
                Bullet::target_layer(emitter.target),
                Damage::new(BULLET_DAMAGE * mods.damage),
            ));

            commands.spawn((
                BasicBullet,
                LinearVelocity(polarity.to_vec2() * BULLET_SPEED * mods.speed),
                {
                    new_transform.translation.x += emitter.width;
                    new_transform
                },
                Bullet::target_layer(emitter.target),
                Damage::new(BULLET_DAMAGE * mods.damage),
            ));

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                PitchRange(BULLET_PITCH_RANGE),
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
#[require(Transform, BulletModifiers, TurnSpeed, Polarity)]
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
            &BulletModifiers,
            &TurnSpeed,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
            Option<&MaxLifetime>,
        )>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, mods, turn_speed, polarity, child_of, transform, lifetime) in
            emitters.iter_mut()
        {
            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let duration = Duration::from_secs_f32(MISSILE_RATE * 1. / mods.rate);
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
            let mut bullet = commands.spawn((
                Missile,
                LinearVelocity::default(),
                Homing::<T>::new(),
                HomingRotate,
                *turn_speed,
                Heading {
                    speed: MISSILE_SPEED * mods.speed,
                    direction,
                },
                new_transform,
                Bullet::target_layer(emitter.target),
                Damage::new(MISSILE_DAMAGE * mods.damage),
            ));

            if let Some(lifetime) = lifetime {
                bullet.insert(Lifetime(Timer::new(lifetime.0, TimerMode::Once)));
            }

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/bullet.wav")),
                PitchRange(BULLET_PITCH_RANGE),
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
#[require(Transform, BulletModifiers, Visibility, Polarity)]
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
            &BulletModifiers,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
            &Children,
        )>,
        mut child: Query<&mut Transform>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut targets: Query<(&mut Health, &GlobalTransform, Option<&Player>)>,
        mut writer: EventWriter<BulletCollisionEvent>,
        mut commands: Commands,
    ) -> Result {
        let delta = time.delta();

        for (entity, emitter, timer, mods, polarity, child_of, transform, children) in
            emitters.iter_mut()
        {
            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

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
                    child.scale.x += difference.max(LASER_SPEED) * delta.as_secs_f32() * mods.speed;
                }

                child.translation.y = direction.y * (4.0 + child.scale.x * 8.0 / 2.0);

                if let Ok((mut target, target_transform, player)) = targets.get_mut(hit_data.entity)
                {
                    if (child.scale.x * 8.0 - hit_data.distance).abs() <= 16.0 {
                        target.damage(15.0 * mods.damage * delta.as_secs_f32());
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
                                player.is_some(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Component, Default)]
#[require(Transform, BulletModifiers, Polarity, Visibility::Hidden)]
pub struct MineEmitter(Layer);

impl MineEmitter {
    pub fn player() -> Self {
        Self(Layer::Player)
    }

    pub fn enemy() -> Self {
        Self(Layer::Enemy)
    }
}

impl MineEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &MineEmitter,
            Option<&mut BulletTimer>,
            &BulletModifiers,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<Option<&BulletModifiers>>,
        player: Single<&Transform, With<Player>>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (entity, emitter, timer, mods, polarity, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let duration = Duration::from_secs_f32(MINE_RATE * 1. / mods.rate);
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

            let to_player =
                (player.translation.xy() - new_transform.translation.xy()).normalize_or(Vec2::ONE);
            let velocity = polarity.to_vec2().with_x(0.5)
                * to_player.xy().with_y(-to_player.y)
                * MINE_SPEED
                * mods.speed;
            commands.spawn((
                Mine,
                LinearVelocity(velocity),
                new_transform,
                Bullet::target_layer(emitter.0),
                Damage::new(MINE_DAMAGE * mods.damage),
            ));

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/mine.wav")),
                PlaybackSettings {
                    volume: Volume::Decibels(-18.0),
                    ..PlaybackSettings::ONCE
                },
                sample_effects![BandPassNode::new(1000.0, 4.0)],
            ));
        }
    }
}

#[derive(Component)]
struct PulseTimer {
    wait: Timer,
    bullet: Timer,
    pulses: usize,
    state: PulseState,
}

enum PulseState {
    Wait,
    Bullet(usize),
}

impl PulseTimer {
    pub fn new(wait: f32, bullet: f32, pulses: usize) -> Self {
        assert!(pulses > 1, "just use a normal bullet timer!");
        Self {
            wait: Timer::from_seconds(wait, TimerMode::Repeating),
            bullet: Timer::from_seconds(bullet, TimerMode::Repeating),
            state: PulseState::Wait,
            pulses,
        }
    }

    pub fn just_finished(&mut self, time: &Time) -> bool {
        let delta = time.delta();

        match self.state {
            PulseState::Wait => {
                self.wait.tick(delta);

                let finished = self.wait.just_finished();
                if finished {
                    self.state = PulseState::Bullet(1);
                    self.bullet.reset();
                }
                finished
            }
            PulseState::Bullet(count) => {
                self.bullet.tick(delta);

                let finished = self.bullet.just_finished();
                if finished {
                    self.state = PulseState::Bullet(count + 1);
                    if count >= self.pulses {
                        self.state = PulseState::Wait;
                        self.wait.reset();
                    }
                }
                finished
            }
        }
    }

    pub fn current_pulse(&self) -> usize {
        match self.state {
            PulseState::Wait => 0,
            PulseState::Bullet(pulse) => pulse,
        }
    }
}

#[derive(Component)]
#[require(Transform, BulletModifiers, Polarity, Visibility::Hidden)]
pub struct OrbEmitter(Layer);

impl OrbEmitter {
    pub fn player() -> Self {
        Self(Layer::Player)
    }

    pub fn enemy() -> Self {
        Self(Layer::Enemy)
    }
}

impl OrbEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &mut OrbEmitter,
            Option<&mut PulseTimer>,
            &BulletModifiers,
            &Polarity,
            &ChildOf,
            &GlobalTransform,
        )>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        for (entity, emitter, timer, mods, polarity, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(PulseTimer::new(
                    ORB_WAIT_RATE * 1. / mods.rate,
                    ORB_SHOT_RATE * 1. / mods.rate,
                    ORB_WAVES,
                ));
                continue;
            };

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.just_finished(&time) {
                continue;
            }

            let bullets = 10;
            for angle in 0..bullets {
                let angle = (angle as f32 / bullets as f32) * 2. * std::f32::consts::PI
                    + timer.current_pulse() as f32 * std::f32::consts::PI / 4.;
                commands.spawn((
                    Orb,
                    LinearVelocity(Vec2::from_angle(angle) * ORB_SPEED * mods.speed),
                    new_transform,
                    Bullet::target_layer(emitter.0),
                    Damage::new(ORB_DAMAGE * mods.damage),
                ));
            }

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/orb.wav")),
                PlaybackSettings {
                    volume: Volume::Decibels(-18.0),
                    ..PlaybackSettings::ONCE
                },
            ));
        }
    }
}
