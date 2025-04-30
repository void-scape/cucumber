use crate::{
    Layer,
    animation::{AnimationController, AnimationIndices, AnimationMode},
    assets::{self, MISC_PATH, MiscLayout},
    auto_collider::ImageCollider,
    bounds::ScreenBounds,
    enemy::Enemy,
    health::{Damage, Health},
    player::Player,
};
use avian2d::prelude::*;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_optix::shake::TraumaCommands;
use bevy_seedling::{
    prelude::Volume,
    sample::{PitchRange, PlaybackSettings, SamplePlayer},
};
use std::time::Duration;
use strum_macros::EnumIter;

pub mod emitter;
pub mod homing;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum BulletSystems {
    Collision,
    Lifetime,
    Sprite,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((emitter::EmitterPlugin, homing::HomingPlugin))
            .add_event::<BulletCollisionEvent>()
            .add_systems(
                PostUpdate,
                (handle_enemy_collision, handle_player_collision).in_set(BulletSystems::Collision),
            )
            .add_systems(
                Update,
                (
                    (manage_lifetime, kill_on_wall).in_set(BulletSystems::Lifetime),
                    bullet_collision_effects,
                ),
            )
            .add_systems(
                PostUpdate,
                (init_bullet_sprite.in_set(BulletSystems::Sprite),).chain(),
            );
    }
}

#[derive(Default, Component, Clone, Copy)]
pub enum Polarity {
    North,
    #[default]
    South,
}

impl Polarity {
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Self::North => Vec2::Y,
            Self::South => Vec2::NEG_Y,
        }
    }
}

#[derive(Clone, Copy, Default, EnumIter, Component)]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    #[default]
    South,
    SouthWest,
    West,
}

impl Direction {
    pub fn to_vec2(self) -> Vec2 {
        match self {
            Self::NorthWest => Vec2::new(-1., 1.).normalize(),
            Self::North => Vec2::Y,
            Self::NorthEast => Vec2::ONE.normalize(),
            Self::East => Vec2::X,
            Self::SouthEast => Vec2::new(1., -1.).normalize(),
            Self::South => Vec2::NEG_Y,
            Self::SouthWest => Vec2::NEG_ONE.normalize(),
            Self::West => Vec2::NEG_X,
        }
    }
}
/// The rate at which bullets should fire.
///
/// This doesn't have any particular unit;
/// emitters can interpret this however they like.
#[derive(Component, Clone, Copy)]
pub struct BulletRate(pub f32);

impl Default for BulletRate {
    fn default() -> Self {
        Self(1.0)
    }
}

/// The speed at which bullets should travel.
///
/// This doesn't have any particular unit;
/// emitters can interpret this however they like.
#[derive(Component, Clone, Copy)]
pub struct BulletSpeed(pub f32);

impl Default for BulletSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

fn init_bullet_sprite(
    mut commands: Commands,
    bullets: Query<(Entity, &BulletSprite, &LinearVelocity), Without<Sprite>>,
    server: Res<AssetServer>,
) {
    for (entity, sprite, velocity) in bullets.iter() {
        let mut sprite = assets::sprite_rect8(&server, sprite.path, sprite.cell);
        sprite.flip_y = velocity.0.y < 0.;
        sprite.flip_x = velocity.0.x < 0.;
        commands.entity(entity).insert(sprite);
    }
}

#[derive(Component)]
pub struct BulletTimer {
    pub timer: Timer,
}

#[derive(Debug, Component)]
pub struct Lifetime(pub Timer);

impl Default for Lifetime {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(10), TimerMode::Once))
    }
}

fn manage_lifetime(mut q: Query<(Entity, &mut Lifetime)>, time: Res<Time>, mut commands: Commands) {
    let delta = time.delta();

    for (entity, mut lifetime) in q.iter_mut() {
        lifetime.0.tick(delta);

        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Copy, Component, Default)]
#[require(Polarity, BulletSpeed, Sensor, RigidBody::Kinematic)]
pub struct Bullet;

impl Bullet {
    pub fn target_layer(target: Layer) -> CollisionLayers {
        CollisionLayers::new([Layer::Bullet], [target, Layer::Bounds])
    }
}

fn kill_on_wall(
    mut commands: Commands,
    bounds: Query<&CollidingEntities, With<ScreenBounds>>,
    bullets: Query<Entity, With<Bullet>>,
) {
    for colliding_entities in bounds.iter() {
        for entity in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| bullets.get(entity))
        {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Bullet)]
#[component(on_add = Self::on_add_hook)]
pub enum BulletType {
    Basic,
    Common,
}

impl BulletType {
    fn on_add_hook(mut world: DeferredWorld, ctx: HookContext) {
        let ty = *world.get::<BulletType>(ctx.entity).unwrap();

        match ty {
            BulletType::Basic => {
                world.commands().entity(ctx.entity).insert(BasicBullet);
            }
            BulletType::Common => {
                world.commands().entity(ctx.entity).insert(CommonBullet);
            }
        }
    }
}

#[derive(Component)]
#[require(ImageCollider)]
pub struct BulletSprite {
    path: &'static str,
    cell: UVec2,
}

impl BulletSprite {
    pub fn from_cell(x: u32, y: u32) -> Self {
        Self {
            path: assets::PROJECTILES_PATH,
            cell: UVec2::new(x, y),
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(BulletSprite::from_cell(0, 1))]
pub struct BasicBullet;

#[derive(Clone, Copy, Component)]
#[require(BulletSprite::from_cell(2, 1))]
pub struct CommonBullet;

fn handle_enemy_collision(
    bullets: Query<(Entity, &Damage, &BulletSprite, &GlobalTransform), With<Bullet>>,
    mut enemies: Query<(&CollidingEntities, &mut Health), With<Enemy>>,
    mut commands: Commands,
    mut writer: EventWriter<BulletCollisionEvent>,
) {
    for (colliding_entities, mut health) in enemies.iter_mut() {
        for (bullet, damage, sprite, transform) in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| bullets.get(entity))
        {
            health.damage(**damage);
            writer.write(BulletCollisionEvent::new(
                sprite.cell,
                transform.compute_transform(),
                BulletSource::Player,
            ));
            commands.entity(bullet).despawn();
        }
    }
}

fn handle_player_collision(
    bullets: Query<(Entity, &Damage, &BulletSprite, &GlobalTransform), With<Bullet>>,
    player: Single<(&CollidingEntities, &mut Health), With<Player>>,
    mut commands: Commands,
    mut writer: EventWriter<BulletCollisionEvent>,
) {
    let (colliding_entities, mut health) = player.into_inner();

    for (bullet, damage, sprite, transform) in colliding_entities
        .iter()
        .copied()
        .flat_map(|entity| bullets.get(entity))
    {
        health.damage(**damage);
        writer.write(BulletCollisionEvent::new(
            sprite.cell,
            transform.compute_transform(),
            BulletSource::Enemy,
        ));
        commands.entity(bullet).despawn();
    }
}

#[derive(Event)]
pub struct BulletCollisionEvent {
    /// The cell in the projectile spritesheet.
    pub cell: UVec2,
    pub transform: Transform,
    pub source: BulletSource,
}

impl BulletCollisionEvent {
    pub fn new(cell: UVec2, mut transform: Transform, source: BulletSource) -> Self {
        transform.translation = transform.translation.round();
        transform.translation.z = 1.;
        Self {
            cell,
            transform,
            source,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BulletSource {
    Player,
    Enemy,
}

fn bullet_collision_effects(
    mut commands: Commands,
    mut reader: EventReader<BulletCollisionEvent>,
    server: Res<AssetServer>,
    misc_layout: Res<MiscLayout>,
) {
    for event in reader.read() {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/melee.wav")),
            PitchRange(0.98..1.02),
            PlaybackSettings {
                volume: Volume::Decibels(-32.0),
                ..PlaybackSettings::ONCE
            },
        ));

        commands.spawn((
            event.transform,
            Sprite::from_atlas_image(
                server.load(MISC_PATH),
                TextureAtlas::from(misc_layout.0.clone()),
            ),
            AnimationController::from_seconds(
                AnimationIndices::new(AnimationMode::Despawn, 83..=86),
                0.05,
            ),
        ));

        match event.source {
            BulletSource::Enemy => commands.add_trauma(0.15),
            BulletSource::Player => commands.add_trauma(0.04),
        }
    }
}
