use super::{
    BasicBullet, BlueOrb, Bullet, BulletCollisionEvent, BulletSource, BulletSprite, BulletTimer,
    ColorMod, Lifetime, MaxLifetime, Missile, Polarity,
    homing::{Heading, Homing, TurnSpeed},
    player,
};
use crate::{
    Avian, DespawnRestart, HEIGHT, Layer,
    bullet::PlayerBullet,
    enemy::{Enemy, arcs::ArcsEmitter, minethrower::MineEmitter},
    health::{Damage, DamageEvent, Health, HealthSet},
    particles::{self, ParticleAppExt, ParticleBundle, ParticleEmitter, ParticleState},
    player::Player,
    sprites::{self, CellSize},
};
use avian2d::prelude::*;
use bevy::{
    ecs::{
        component::{HookContext, Mutable},
        world::DeferredWorld,
    },
    prelude::*,
    time::Stopwatch,
};
use bevy_seedling::prelude::*;
use rand::seq::IteratorRandom;
use std::{f32::consts::PI, marker::PhantomData, time::Duration};
use strum::IntoEnumIterator;

pub const PLAYER_BULLET_SPEED: f32 = 300.;
pub const PLAYER_MISSILE_SPEED: f32 = PLAYER_BULLET_SPEED * 0.8;
pub const BULLET_SPEED: f32 = 75.;
pub const MISSILE_SPEED: f32 = 65.;
pub const LASER_SPEED: f32 = 15.;
pub const ORB_SPEED: f32 = 75.;

pub const PLAYER_BULLET_RATE: f32 = 0.2;
pub const MISSILE_RATE: f32 = 0.5;

const ORB_WAIT_RATE: f32 = 2.;
const ORB_SHOT_RATE: f32 = 0.2;
const ORB_WAVES: usize = 8;

pub const MISSILE_HEALTH: f32 = 2.;
pub const MINE_HEALTH: f32 = 1.5;

const BULLET_PITCH_RANGE: core::ops::Range<f64> = 0.9..1.1;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum EmitterSystems {
    Init,
    Update,
}

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EmitterSample>()
            .register_particle_state::<EmitterState>()
            .configure_sets(
                PreUpdate,
                EmitterSystems::Init.before(EmitterSystems::Update),
            )
            .add_systems(
                PreUpdate,
                (
                    (tick_emitter_delay, limit),
                    target_player,
                    (
                        GattlingEmitter::shoot_bullets,
                        player::PlayerGattlingEmitter::shoot_bullets,
                        player::PlayerFocusEmitter::shoot_bullets,
                        BackgroundGattlingEmitter::shoot_bullets,
                        MissileEmitter::shoot_bullets,
                        HomingEmitter::<Enemy>::shoot_bullets,
                        HomingEmitter::<Player>::shoot_bullets,
                        MineEmitter::shoot_bullets,
                        SpiralOrbEmitter::shoot_bullets,
                        ProximityEmitter::shoot_bullets,
                    )
                        .in_set(EmitterSystems::Update),
                    play_samples,
                )
                    .chain(),
            )
            .add_systems(Avian, LaserEmitter::laser.before(HealthSet));
    }
}

pub trait EmitterAppExt {
    fn add_emitter_system<T: ShootEmitter>(&mut self) -> &mut Self;
}

impl EmitterAppExt for App {
    fn add_emitter_system<T: ShootEmitter>(&mut self) -> &mut Self {
        self.add_systems(
            PreUpdate,
            (
                //init_emitter::<T>.in_set(EmitterSystems::Init),
                update_emitter::<T>.in_set(EmitterSystems::Update),
            ),
        )
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct EmitterState {
    pub enabled: bool,
}

impl ParticleState for EmitterState {
    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for EmitterState {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Event)]
pub struct EmitterSample(pub EmitterBullet);

pub enum EmitterBullet {
    Bullet,
    Missile,
    Mine,
    Orb,
    Arrow,
}

fn play_samples(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<EmitterSample>,
) {
    for event in reader.read() {
        match event.0 {
            EmitterBullet::Bullet => {
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
            EmitterBullet::Orb => {
                commands.spawn((
                    SamplePlayer::new(server.load("audio/sfx/orb.wav")),
                    PlaybackSettings {
                        volume: Volume::Linear(0.5),
                        ..PlaybackSettings::ONCE
                    },
                ));
            }
            EmitterBullet::Mine => {
                commands.spawn((
                    SamplePlayer::new(server.load("audio/sfx/mine.wav")),
                    PlaybackSettings {
                        volume: Volume::Decibels(-18.0),
                        ..PlaybackSettings::ONCE
                    },
                    sample_effects![BandPassNode::new(1000.0, 4.0)],
                ));
            }
            EmitterBullet::Arrow => {
                //commands.spawn((
                //    SamplePlayer::new(server.load("audio/sfx/bfxr/arrow.wav")),
                //    PlaybackSettings {
                //        volume: Volume::Decibels(-18.0),
                //        ..PlaybackSettings::ONCE
                //    },
                //));
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component)]
pub struct EmitterDelay(Timer);

impl EmitterDelay {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

pub fn tick_emitter_delay(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut EmitterDelay)>,
) {
    for (entity, mut delay) in entities.iter_mut() {
        delay.0.tick(time.delta());
        if delay.0.finished() {
            commands.entity(entity).remove::<EmitterDelay>();
        }
    }
}

#[derive(Component)]
#[require(Shots)]
pub struct ShotLimit(pub usize);

#[derive(Component)]
#[require(Pulses)]
pub struct PulseLimit(pub usize);

#[derive(Default, Component)]
struct Shots(usize);

#[derive(Default, Component)]
struct Pulses(usize);

fn limit(
    mut shots: Query<(&mut EmitterState, &Shots, &ShotLimit)>,
    mut pulses: Query<(&mut EmitterState, &Pulses, &PulseLimit), Without<Shots>>,
) {
    for (mut state, shots, limit) in shots.iter_mut() {
        state.enabled = shots.0 < limit.0;
    }

    for (mut state, pulses, limit) in pulses.iter_mut() {
        state.enabled = pulses.0 < limit.0;
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
    pub rate: Rate,
}

impl Default for BulletModifiers {
    fn default() -> Self {
        Self {
            speed: 1.,
            damage: 1.,
            rate: Rate::Factor(1.),
        }
    }
}

impl BulletModifiers {
    pub fn join(&self, other: &Self) -> Self {
        let rate = match self.rate {
            Rate::Secs(secs) => match other.rate {
                Rate::Secs(_) => {
                    panic!("tried to join two second rate modifiers");
                }
                Rate::Factor(factor) => Rate::Secs(secs * factor),
            },
            Rate::Factor(factor) => match other.rate {
                Rate::Secs(secs) => Rate::Secs(secs * factor),
                Rate::Factor(f) => Rate::Factor(factor * f),
            },
        };

        Self {
            speed: self.speed * other.speed,
            damage: self.damage * other.damage,
            rate,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Rate {
    Factor(f32),
    Secs(f32),
}

impl Rate {
    pub fn add_factor(&mut self, factor: f32) {
        match self {
            Self::Factor(f) => *f += factor,
            Self::Secs(secs) => *secs += *secs * factor,
        }
    }

    pub fn duration(self, base: f32) -> Duration {
        match self {
            Rate::Factor(factor) => Duration::from_secs_f32(base * 1. / factor),
            Rate::Secs(secs) => Duration::from_secs_f32(secs),
        }
    }
}

#[derive(Clone, Copy, Component)]
pub enum Target {
    Player { target: Vec2, update: bool },
    Direction(Vec2),
}

impl Target {
    pub const NEG_Y: Self = Self::Direction(Vec2::NEG_Y);

    pub fn player() -> Self {
        Self::Player {
            target: Vec2::NEG_Y,
            update: true,
        }
    }

    pub fn dir(dir: Vec2) -> Self {
        Self::Direction(dir.normalize_or_zero())
    }

    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Self::Player { target, .. } => *target,
            Self::Direction(v) => *v,
        }
    }

    pub fn enable(&mut self, enable: bool) {
        match self {
            Self::Player { update, .. } => *update = enable,
            _ => {}
        }
    }
}

fn target_player(
    player: Single<&Transform, With<Player>>,
    mut entities: Query<(Option<&EmitterState>, &GlobalTransform, &mut Target)>,
) {
    for (_, gt, mut target) in entities
        .iter_mut()
        .filter(|(state, _, _)| state.is_none_or(|state| state.enabled))
    {
        if let Target::Player { target, update } = &mut *target {
            if *update {
                *target = (player.translation.xy() - gt.translation().xy()).normalize_or_zero();
            }
        }
    }
}

#[derive(Default, Component)]
#[require(
    EmitterState,
    BulletModifiers,
    Target::NEG_Y,
    BulletSpeed(Vec2::splat(100.))
)]
pub struct Emitter;

#[derive(Component)]
pub struct BulletSpeed(pub Vec2);

impl BulletSpeed {
    pub fn new(speed: f32) -> BulletSpeed {
        Self(Vec2::splat(speed))
    }
}

pub struct EmitterCtx<'a, T> {
    pub mods: &'a BulletModifiers,
    pub timer: &'a T,
    pub target: &'a Target,
    pub player_position: Vec2,
}

pub trait ShootEmitter: Sized + Component {
    type Timer: EmitterTimer;

    fn timer(&self, mods: &BulletModifiers) -> Self::Timer;

    fn spawn_bullets(
        &self,
        commands: BulletCommands,
        transform: Transform,
        ctx: EmitterCtx<Self::Timer>,
    );

    fn sample() -> Option<EmitterBullet> {
        None
    }
}

pub trait EmitterTimer: Component<Mutability = Mutable> {
    fn tick(&mut self, time: &Time);
    fn shoot(&self) -> bool;
    fn finished_pulse(&self) -> bool;
}

impl EmitterTimer for BulletTimer {
    fn tick(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }

    fn shoot(&self) -> bool {
        self.timer.just_finished()
    }

    fn finished_pulse(&self) -> bool {
        self.timer.just_finished()
    }
}

impl EmitterTimer for PulseTimer {
    fn tick(&mut self, time: &Time) {
        self.tick(time);
    }

    fn shoot(&self) -> bool {
        self.just_finished()
    }

    fn finished_pulse(&self) -> bool {
        self.just_finished() && self.is_waiting()
    }
}

pub trait PulseTime {
    fn wait_time(&self) -> f32;
    fn shot_time(&self) -> f32;
    fn pulses(&self) -> usize;

    fn total_time(&self) -> f32 {
        self.total_shoot_time() + self.wait_time()
    }

    fn total_shoot_time(&self) -> f32 {
        (self.pulses() - 1) as f32 * self.shot_time()
    }
}

#[derive(Component)]
pub struct PulseTimer {
    pub wait: Timer,
    pub bullet: Timer,
    pub pulses: usize,
    pub state: PulseState,
    shoot: bool,
}

pub enum PulseState {
    Wait,
    Bullet(usize),
}

impl PulseTimer {
    pub fn new(rate: Rate, wait: f32, bullet: f32, pulses: usize) -> Self {
        assert!(pulses > 1, "just use a normal bullet timer!");

        let (wait, bullet) = match rate {
            Rate::Factor(factor) => (wait * factor, bullet * factor),
            Rate::Secs(secs) => {
                let ratio = wait / (wait + bullet);
                (secs * ratio, secs - secs * ratio)
            }
        };

        Self {
            wait: Timer::from_seconds(wait, TimerMode::Repeating),
            bullet: Timer::from_seconds(bullet, TimerMode::Repeating),
            state: PulseState::Wait,
            pulses,
            shoot: false,
        }
    }

    pub fn ready(rate: Rate, wait: f32, bullet: f32, pulses: usize) -> Self {
        let mut slf = Self::new(rate, wait, bullet, pulses);
        slf.reset_active();
        slf
    }

    pub fn is_waiting(&self) -> bool {
        matches!(self.state, PulseState::Wait)
    }

    pub fn tick(&mut self, time: &Time) {
        let delta = time.delta();

        match self.state {
            PulseState::Wait => {
                self.wait.tick(delta);

                let finished = self.wait.just_finished();
                if finished {
                    self.state = PulseState::Bullet(1);
                    self.bullet.reset();
                }
                self.shoot = finished;
            }
            PulseState::Bullet(count) => {
                self.bullet.tick(delta);

                let finished = self.bullet.just_finished();
                if finished {
                    self.state = PulseState::Bullet(count + 1);
                    if count + 1 >= self.pulses {
                        self.state = PulseState::Wait;
                        self.wait.reset();
                    }
                }
                self.shoot = finished;
            }
        }
    }

    pub fn just_finished(&self) -> bool {
        self.shoot
    }

    pub fn reset_active(&mut self) {
        self.state = PulseState::Wait;
        self.wait.set_elapsed(self.wait.duration());
    }

    pub fn current_pulse(&self) -> usize {
        match self.state {
            PulseState::Wait => 0,
            PulseState::Bullet(pulse) => pulse,
        }
    }
}

impl PulseTime for PulseTimer {
    fn wait_time(&self) -> f32 {
        self.wait.duration().as_secs_f32()
    }

    fn shot_time(&self) -> f32 {
        self.bullet.duration().as_secs_f32()
    }

    fn pulses(&self) -> usize {
        self.pulses
    }
}

pub struct BulletCommands<'a, 'c> {
    pub commands: &'a mut Commands<'c, 'c>,
    mods: BulletModifiers,
    target: Target,
    speed: Vec2,
}

impl<'a, 'c> BulletCommands<'a, 'c> {
    fn new(
        commands: &'a mut Commands<'c, 'c>,
        mods: BulletModifiers,
        target: Target,
        speed: Vec2,
    ) -> Self {
        Self {
            commands,
            mods,
            target,
            speed,
        }
    }

    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let mut bullet = self.commands.spawn(bundle);
        bullet.insert((
            Damage::new(1. * self.mods.damage),
            LinearVelocity(self.target.as_vec2() * self.speed * self.mods.speed),
        ));
        bullet
    }

    pub fn spawn_angled(&mut self, angle_offset: f32, bundle: impl Bundle) -> EntityCommands<'_> {
        let mut bullet = self.commands.spawn(bundle);
        bullet.insert((
            Damage::new(1. * self.mods.damage),
            LinearVelocity(
                Vec2::from_angle(angle_offset)
                    .rotate(self.target.as_vec2())
                    .normalize_or_zero()
                    * self.speed
                    * self.mods.speed,
            ),
        ));
        bullet
    }

    pub fn spawn_naked(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let mut bullet = self.commands.spawn(bundle);
        bullet.insert(Damage::new(1. * self.mods.damage));
        bullet
    }
}

pub trait RotateBullet {
    fn look_at(&mut self, target: &Target) -> &mut Self;

    fn look_at_offset(&mut self, target: &Target, offset: f32) -> &mut Self;
}

impl RotateBullet for EntityCommands<'_> {
    fn look_at(&mut self, target: &Target) -> &mut Self {
        let radians = target.as_vec2().to_angle();
        self.insert(Rotation::radians(radians))
    }

    fn look_at_offset(&mut self, target: &Target, offset: f32) -> &mut Self {
        let radians = target.as_vec2().to_angle();
        self.insert(Rotation::radians(radians + offset))
    }
}

//fn init_emitter<T: ShootEmitter>(
//    mut commands: Commands,
//    emitters: Query<
//        (Entity, &T),
//        (
//            Without<T::Timer>,
//            With<EmitterState>,
//            With<BulletModifiers>,
//            With<BulletSpeed>,
//            With<Target>,
//        ),
//    >,
//) {
//    for (entity, emitter) in emitters.iter() {
//        commands.entity(entity).insert(T::timer(emitter));
//    }
//}

fn update_emitter<T: ShootEmitter>(
    mut commands: Commands,
    mut emitters: Query<
        (
            Entity,
            &T,
            &EmitterState,
            Option<&mut T::Timer>,
            &BulletModifiers,
            &BulletSpeed,
            &Target,
            Option<&ChildOf>,
            &GlobalTransform,
            Option<&mut Shots>,
            Option<&mut Pulses>,
        ),
        Without<EmitterDelay>,
    >,
    player: Single<&Transform, With<Player>>,
    parents: Query<Option<&BulletModifiers>>,
    time: Res<Time>,
    mut writer: EventWriter<EmitterSample>,
) {
    for (entity, emitter, state, timer, mods, speed, target, child_of, gt, shots, pulses) in
        emitters.iter_mut()
    {
        if !state.enabled {
            continue;
        }

        let mods = if let Some(Ok(Some(parent_mods))) =
            child_of.map(|child_of| parents.get(child_of.parent()))
        {
            parent_mods.join(mods)
        } else {
            *mods
        };

        let Some(mut timer) = timer else {
            commands.entity(entity).insert(T::timer(emitter, &mods));
            continue;
        };

        timer.tick(&time);
        if timer.finished_pulse() {
            if let Some(mut pulses) = pulses {
                pulses.0 += 1;
            }
        }
        if !timer.shoot() {
            continue;
        }

        if let Some(mut shots) = shots {
            shots.0 += 1;
        }

        T::spawn_bullets(
            emitter,
            BulletCommands::new(&mut commands.reborrow(), mods, *target, speed.0),
            gt.compute_transform(),
            EmitterCtx {
                player_position: player.translation.xy(),
                mods: &mods,
                timer: &timer,
                target,
            },
        );

        if let Some(sample) = T::sample() {
            writer.write(EmitterSample(sample));
        }
    }
}

#[derive(Component)]
#[require(
    Transform,
    BulletModifiers,
    Polarity,
    Visibility::Hidden,
    ProximityTimer
)]
pub struct ProximityEmitter;

#[derive(Component, Default)]
struct ProximityTimer(Stopwatch);

impl ProximityEmitter {
    fn shoot_bullets(
        mut emitters: Query<
            (
                &Self,
                &mut ProximityTimer,
                &BulletModifiers,
                &Polarity,
                &ChildOf,
                &GlobalTransform,
            ),
            Without<EmitterDelay>,
        >,
        player: Single<&GlobalTransform, With<Player>>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) -> Result {
        let delta = time.delta();
        let player = player.into_inner().compute_transform();

        for (_emitter, mut timer, mods, polarity, child_of, transform) in emitters.iter_mut() {
            let parent_mods = parents.get(child_of.parent())?;
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if timer.0.tick(delta).elapsed() < Duration::from_secs_f32(1.) {
                continue;
            }

            let proximity = new_transform.translation.x - player.translation.x;
            if proximity.abs() > 20. {
                continue;
            }

            timer.0.reset();

            commands.spawn((
                BasicBullet,
                LinearVelocity(polarity.to_vec2() * BULLET_SPEED * mods.speed),
                new_transform,
            ));

            writer.write(EmitterSample(EmitterBullet::Bullet));
        }

        Ok(())
    }
}

#[derive(Component)]
#[require(
    Transform,
    BulletModifiers,
    EmitterState,
    //ParticleBundle<EmitterState> = Self::particles()
)]
#[component(on_add = Self::insert_timer)]
pub struct GattlingEmitter(pub f32);

impl Default for GattlingEmitter {
    fn default() -> Self {
        GattlingEmitter(0.1)
    }
}

impl GattlingEmitter {
    fn particles() -> ParticleBundle<EmitterState> {
        ParticleBundle::<EmitterState>::from_emitter(
            ParticleEmitter::from_effect("particles/bullet_shells.ron")
                .with_sprite("shell.png")
                .with(particles::transform(
                    Transform::from_xyz(0., -5., -1.).with_rotation(Quat::from_rotation_z(PI)),
                )),
        )
    }

    fn insert_timer(mut world: DeferredWorld, ctx: HookContext) {
        let mods = world.get::<BulletModifiers>(ctx.entity).unwrap();
        let duration = mods.rate.duration(PLAYER_BULLET_RATE);
        world.commands().entity(ctx.entity).insert(BulletTimer {
            timer: Timer::new(duration, TimerMode::Repeating),
        });
    }
}

impl GattlingEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &EmitterState,
            &GattlingEmitter,
            &mut BulletTimer,
            &BulletModifiers,
            &GlobalTransform,
            &ChildOf,
        )>,
        parents: Query<Option<&BulletModifiers>, With<Children>>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (_entity, state, emitter, mut timer, mods, transform, child_of) in emitters.iter_mut() {
            if !state.enabled {
                continue;
            }

            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += Vec3::Y * 4.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            let duration = mods.rate.duration(PLAYER_BULLET_RATE);
            timer.timer.set_duration(duration);

            commands.spawn((
                BasicBullet,
                PlayerBullet,
                ColorMod::Blue,
                LinearVelocity(
                    (Vec2::Y - Vec2::new(emitter.0, 0.)).normalize()
                        * PLAYER_BULLET_SPEED
                        * mods.speed,
                ),
                {
                    let mut t = new_transform;
                    t.translation.x -= 5.;
                    t
                },
                Bullet::target_layer(Layer::Enemy),
            ));

            commands.spawn((
                BasicBullet,
                PlayerBullet,
                ColorMod::Blue,
                LinearVelocity(Vec2::Y * PLAYER_BULLET_SPEED * mods.speed),
                new_transform,
                Bullet::target_layer(Layer::Enemy),
            ));

            commands.spawn((
                BasicBullet,
                PlayerBullet,
                ColorMod::Blue,
                LinearVelocity(
                    (Vec2::Y + Vec2::new(emitter.0, 0.)).normalize()
                        * PLAYER_BULLET_SPEED
                        * mods.speed,
                ),
                {
                    new_transform.translation.x += 5.;
                    new_transform
                },
                Bullet::target_layer(Layer::Enemy),
            ));
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility, BulletModifiers)]
#[component(on_add = Self::insert_timer)]
pub struct BackgroundGattlingEmitter(pub f32, pub Vec2);

impl Default for BackgroundGattlingEmitter {
    fn default() -> Self {
        BackgroundGattlingEmitter(0.1, Vec2::Y)
    }
}

impl BackgroundGattlingEmitter {
    fn insert_timer(mut world: DeferredWorld, ctx: HookContext) {
        let mods = world.get::<BulletModifiers>(ctx.entity).unwrap();
        let duration = mods.rate.duration(PLAYER_BULLET_RATE);

        let mut rng = rand::rng();
        world.commands().entity(ctx.entity).insert((
            BulletTimer {
                timer: Timer::new(duration, TimerMode::Repeating),
            },
            ColorMod::iter().choose(&mut rng).unwrap(),
        ));
    }
}

impl BackgroundGattlingEmitter {
    fn shoot_bullets(
        mut emitters: Query<(
            Entity,
            &BackgroundGattlingEmitter,
            &mut BulletTimer,
            &BulletModifiers,
            &GlobalTransform,
            &ColorMod,
        )>,
        time: Res<Time>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (_entity, emitter, mut timer, mods, gt, color) in emitters.iter_mut() {
            let mut new_transform = gt.compute_transform();

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            let duration = mods.rate.duration(PLAYER_BULLET_RATE);
            timer.timer.set_duration(duration);

            commands.spawn((
                Lifetime::new(5.),
                DespawnRestart,
                BulletSprite::from_cell(1, 2)
                    .with_brightness(0.3)
                    .with_alpha(0.5),
                *color,
                RigidBody::Kinematic,
                PlayerBullet,
                LinearVelocity(
                    (emitter.1 - Vec2::splat(emitter.0)).normalize()
                        * PLAYER_BULLET_SPEED
                        * mods.speed,
                ),
                {
                    let mut t = new_transform;
                    t.translation.x -= 5.;
                    t
                },
            ));

            commands.spawn((
                Lifetime::new(5.),
                DespawnRestart,
                BulletSprite::from_cell(1, 2)
                    .with_brightness(0.3)
                    .with_alpha(0.5),
                *color,
                RigidBody::Kinematic,
                PlayerBullet,
                LinearVelocity(emitter.1 * PLAYER_BULLET_SPEED * mods.speed),
                new_transform,
            ));

            commands.spawn((
                Lifetime::new(5.),
                DespawnRestart,
                BulletSprite::from_cell(1, 2)
                    .with_brightness(0.3)
                    .with_alpha(0.5),
                *color,
                RigidBody::Kinematic,
                PlayerBullet,
                LinearVelocity(
                    (emitter.1 + Vec2::splat(emitter.0)).normalize()
                        * PLAYER_BULLET_SPEED
                        * mods.speed,
                ),
                {
                    new_transform.translation.x += 5.;
                    new_transform
                },
            ));
        }
    }
}

#[derive(Component, Default)]
#[require(
    Transform,
    BulletModifiers,
    EmitterState,
    ParticleBundle<EmitterState> = Self::particles(),
)]
#[component(on_add = Self::insert_timer)]
pub struct MissileEmitter;

impl MissileEmitter {
    fn particles() -> ParticleBundle<EmitterState> {
        ParticleBundle::<EmitterState>::from_emitter(
            ParticleEmitter::from_effect("particles/bullet_shells.ron")
                .with_sprite("missile_shell.png")
                .with(particles::transform(
                    Transform::from_xyz(0., -5., -1.).with_rotation(Quat::from_rotation_z(PI)),
                )),
        )
    }

    fn insert_timer(mut world: DeferredWorld, ctx: HookContext) {
        let mods = world.get::<BulletModifiers>(ctx.entity).unwrap();
        let duration = mods.rate.duration(MISSILE_RATE);
        world.commands().entity(ctx.entity).insert(BulletTimer {
            timer: Timer::new(duration, TimerMode::Repeating),
        });
    }
}

impl MissileEmitter {
    fn shoot_bullets(
        mut emitters: Query<
            (
                &MissileEmitter,
                &EmitterState,
                &mut BulletTimer,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        targets: Query<&GlobalTransform, With<Enemy>>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (_emitter, state, mut timer, mods, child_of, transform) in emitters.iter_mut() {
            if !state.enabled {
                continue;
            }

            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            let duration = mods.rate.duration(MISSILE_RATE);
            timer.timer.set_duration(duration);

            let w = crate::WIDTH / 2.;
            let h = crate::HEIGHT / 2.;

            let p = transform.translation().xy();
            let target = match targets
                .iter()
                .sort_unstable_by::<&GlobalTransform>(|a, b| {
                    a.translation()
                        .xy()
                        .distance(p)
                        .total_cmp(&b.translation().xy().distance(p))
                })
                .filter(|gt| {
                    gt.translation().x > -w
                        && gt.translation().x < w
                        && gt.translation().y > -h
                        && gt.translation().y < h
                })
                .next()
            {
                Some(gt) => (gt.translation().xy() - p).normalize_or_zero(),
                None => Vec2::Y,
            };

            let new_transform = transform.compute_transform();
            commands.spawn((
                Missile,
                PlayerBullet,
                LinearVelocity(target * PLAYER_MISSILE_SPEED * mods.speed),
                new_transform.with_rotation(Quat::from_rotation_z(
                    target.to_angle() - PI / 2.0 + PI / 4.,
                )),
                Bullet::target_layer(Layer::Enemy),
            ));

            writer.write(EmitterSample(EmitterBullet::Missile));
        }
    }
}

#[derive(Component, Default)]
#[require(Transform, BulletModifiers, TurnSpeed, Polarity)]
#[component(on_add = Self::insert_timer)]
pub struct HomingEmitter<T> {
    target: Layer,
    _filter: PhantomData<fn() -> T>,
}

impl<T> HomingEmitter<T> {
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

    fn insert_timer(mut world: DeferredWorld, ctx: HookContext) {
        let mods = world.get::<BulletModifiers>(ctx.entity).unwrap();
        let duration = mods.rate.duration(MISSILE_RATE);
        world.commands().entity(ctx.entity).insert(BulletTimer {
            timer: Timer::new(duration, TimerMode::Repeating),
        });
    }
}

impl<T: Component> HomingEmitter<T> {
    fn shoot_bullets(
        mut emitters: Query<
            (
                &HomingEmitter<T>,
                &mut BulletTimer,
                &BulletModifiers,
                &TurnSpeed,
                &Polarity,
                &ChildOf,
                &GlobalTransform,
                Option<&MaxLifetime>,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        let delta = time.delta();

        for (emitter, mut timer, mods, turn_speed, polarity, child_of, transform, lifetime) in
            emitters.iter_mut()
        {
            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += polarity.to_vec2().extend(0.0) * 10.0;

            if !timer.timer.tick(delta).just_finished() {
                continue;
            }
            let duration = mods.rate.duration(MISSILE_RATE);
            timer.timer.set_duration(duration);

            let direction = match polarity {
                Polarity::North => PI / 2.0,
                Polarity::South => -PI / 2.0,
            };
            let mut bullet = commands.spawn((
                Missile,
                LinearVelocity::default(),
                Homing::<T>::new(),
                *turn_speed,
                Heading {
                    speed: MISSILE_SPEED * mods.speed,
                    direction,
                },
                new_transform,
                Bullet::target_layer(emitter.target),
            ));

            if let Some(lifetime) = lifetime {
                bullet.insert(Lifetime(Timer::new(lifetime.0, TimerMode::Once)));
            }

            writer.write(EmitterSample(EmitterBullet::Missile));
        }
    }
}

#[derive(Component)]
#[require(Transform, BulletModifiers, Visibility)]
#[component(on_insert = Self::on_insert_hook)]
pub struct LaserEmitter {
    layer: Layer,
    pub dir: Vec2,
}

impl LaserEmitter {
    pub fn new(dir: Vec2) -> Self {
        assert!(dir != Vec2::ZERO);
        Self {
            layer: Layer::Player,
            dir: dir.normalize(),
        }
    }
}

impl LaserEmitter {
    fn on_insert_hook(mut world: DeferredWorld, ctx: HookContext) {
        let server = world.resource();
        let sprite = BulletSprite::from_cell(1, 8);
        let sprite = sprites::sprite_rect(server, sprite.path, CellSize::Eight, sprite.cell);
        world.commands().entity(ctx.entity).with_child(sprite);
    }

    fn laser(
        spatial_query: SpatialQuery,
        mut emitters: Query<
            (
                Entity,
                Ref<LaserEmitter>,
                Option<&mut BulletTimer>,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
                &Children,
            ),
            Without<EmitterDelay>,
        >,
        mut child: Query<&mut Transform>,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        targets: Query<(Entity, &GlobalTransform, Option<&Player>), With<Health>>,
        mut writer: EventWriter<BulletCollisionEvent>,
        mut commands: Commands,
        mut damage_writer: EventWriter<DamageEvent>,
    ) -> Result {
        let delta = time.delta();

        for (entity, emitter, timer, mods, child_of, gt, children) in emitters.iter_mut() {
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

            let direction = emitter.dir;
            let mut new_transform = gt.compute_transform();
            new_transform.translation += direction.extend(0.0) * 10.0;

            let child_entity = children
                .iter()
                .next()
                .ok_or("laser emitter should have child")?;

            let filter = SpatialQueryFilter::default().with_mask([
                emitter.layer,
                Layer::Bounds,
                Layer::Debris,
            ]);

            if let Some(hit_data) = spatial_query.cast_ray(
                new_transform.translation.xy(),
                Dir2::from_xy(direction.x, direction.y).unwrap_or(Dir2::NORTH),
                HEIGHT,
                false,
                &filter,
            ) {
                let mut child = child.get_mut(child_entity)?;
                if emitter.is_changed() {
                    child.rotation = Quat::from_rotation_z(emitter.dir.to_angle());
                }

                let target_scale = (hit_data.distance + 8.0) / 8.0;
                let difference = target_scale - child.scale.x;

                //child.scale.x = target_scale;
                if difference < 0.0 {
                    child.scale.x = target_scale;
                } else {
                    child.scale.x += difference.max(LASER_SPEED) * delta.as_secs_f32() * mods.speed;
                }

                child.translation.y = direction.y * (4.0 + child.scale.x * 8.0 / 2.0);
                child.translation.x = direction.x * (4.0 + child.scale.x * 8.0 / 2.0);

                if let Ok((entity, target_transform, player)) = targets.get(hit_data.entity) {
                    if (child.scale.x * 8.0 - hit_data.distance).abs() <= 16.0 {
                        if emitter.layer == Layer::Enemy {
                            damage_writer.write(DamageEvent {
                                entity,
                                damage: 15.0 * mods.damage * delta.as_secs_f32(),
                            });
                        } else {
                            damage_writer.write(DamageEvent {
                                entity,
                                damage: 1. * mods.damage,
                            });
                        }
                    }

                    if let Some(mut timer) = timer {
                        if timer.timer.tick(delta).just_finished() {
                            writer.write(BulletCollisionEvent::new(
                                target_transform.compute_transform(),
                                match emitter.layer {
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

#[derive(Clone, Copy, Component)]
#[require(Transform, BulletModifiers)]
pub struct SpiralOrbEmitter {
    waves: usize,
    shot_dur: f32,
    wait_dur: f32,
}

impl Default for SpiralOrbEmitter {
    fn default() -> Self {
        Self {
            waves: ORB_WAVES,
            shot_dur: ORB_SHOT_RATE,
            wait_dur: ORB_WAIT_RATE,
        }
    }
}

impl PulseTime for SpiralOrbEmitter {
    fn wait_time(&self) -> f32 {
        self.wait_dur
    }

    fn shot_time(&self) -> f32 {
        self.shot_dur
    }

    fn pulses(&self) -> usize {
        self.waves
    }
}

impl SpiralOrbEmitter {
    pub fn new(waves: usize, wait_dur: f32, shot_dur: f32) -> Self {
        Self {
            waves,
            wait_dur,
            shot_dur,
        }
    }
}

impl SpiralOrbEmitter {
    fn shoot_bullets(
        mut commands: Commands,
        mut emitters: Query<
            (
                Entity,
                &mut SpiralOrbEmitter,
                Option<&mut PulseTimer>,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
            ),
            Without<EmitterDelay>,
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
    ) {
        for (entity, emitter, timer, mods, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let Some(mut timer) = timer else {
                commands.entity(entity).insert(PulseTimer::new(
                    mods.rate,
                    emitter.wait_dur,
                    emitter.shot_dur,
                    emitter.waves,
                ));
                continue;
            };
            continue;

            //if !timer.just_finished(&time) {
            //    continue;
            //}

            let new_transform = transform.compute_transform();
            let bullets = 10;
            for angle in 0..bullets {
                let angle = (angle as f32 / bullets as f32) * 2. * std::f32::consts::PI
                    + timer.current_pulse() as f32 * std::f32::consts::PI / 4.;
                commands.spawn((
                    BlueOrb,
                    LinearVelocity(Vec2::from_angle(angle) * ORB_SPEED * mods.speed),
                    new_transform,
                ));
            }

            writer.write(EmitterSample(EmitterBullet::Orb));
        }
    }
}
