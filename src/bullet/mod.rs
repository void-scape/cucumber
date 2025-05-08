use self::homing::HomingRotate;
use crate::{
    GameState, Layer,
    animation::{AnimationController, AnimationIndices, AnimationMode},
    assets::{self, MISC_PATH, MiscLayout},
    auto_collider::ImageCollider,
    bounds::WallDespawn,
    health::{Damage, Health},
    player::Player,
    tween::{OnEnd, physics_time_mult},
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{RED, SKY_BLUE},
    ecs::{component::HookContext, world::DeferredWorld},
    platform::collections::HashSet,
    prelude::*,
};
use bevy_optix::{
    glitch::{GlitchIntensity, GlitchSettings, glitch_intensity},
    pixel_perfect::OuterCamera,
    post_process::PostProcessCommand,
    shake::TraumaCommands,
};
use bevy_seedling::{
    prelude::Volume,
    sample::{PitchRange, PlaybackSettings, SamplePlayer},
};
use bevy_tween::{
    prelude::{AnimationBuilderExt, EaseKind},
    tween::{IntoTarget, TargetResource},
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
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(
                PostUpdate,
                (handle_destructable_collision, handle_player_collision)
                    .in_set(BulletSystems::Collision),
            )
            .add_systems(
                Update,
                (
                    manage_lifetime.in_set(BulletSystems::Lifetime),
                    bullet_collision_effects,
                ),
            )
            .add_systems(
                PostUpdate,
                init_bullet_sprite.in_set(BulletSystems::Sprite).chain(),
            );
    }
}

fn restart(mut commands: Commands, bullets: Query<Entity, With<Bullet>>) {
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
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

fn init_bullet_sprite(
    mut commands: Commands,
    bullets: Query<
        (
            Entity,
            &BulletSprite,
            &ColorMod,
            Option<&LinearVelocity>,
            Option<&HomingRotate>,
        ),
        Without<Sprite>,
    >,
    server: Res<AssetServer>,
) {
    for (entity, sprite, color, velocity, rotation) in bullets.iter() {
        let mut sprite = assets::sprite_rect8(&server, sprite.path, sprite.cell);
        if rotation.is_none() {
            if let Some(velocity) = velocity {
                sprite.flip_y = velocity.0.y < 0.;
                sprite.flip_x = velocity.0.x < 0.;
            }
        }
        match color {
            ColorMod::Enemy => {
                sprite.color = RED.into();
            }
            ColorMod::Friendly => {
                sprite.color = SKY_BLUE.into();
            }
        }
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

fn manage_lifetime(
    mut q: Query<(Entity, &mut Lifetime)>,
    time: Res<Time<Physics>>,
    mut commands: Commands,
) {
    let delta = time.delta();

    for (entity, mut lifetime) in q.iter_mut() {
        lifetime.0.tick(delta);

        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Copy, Component, Default)]
#[require(Polarity, Sensor, RigidBody::Kinematic, WallDespawn)]
#[component(on_add = Self::add_color_mod)]
pub struct Bullet;

impl Bullet {
    pub fn target_layer(target: Layer) -> CollisionLayers {
        CollisionLayers::new(Layer::Bullet, [target, Layer::Bounds, Layer::Debris])
    }

    fn add_color_mod(mut world: DeferredWorld, ctx: HookContext) {
        let layers = world.get::<CollisionLayers>(ctx.entity).unwrap();

        let player_mask =
            CollisionLayers::new(Layer::Bullet, [Layer::Player, Layer::Bounds, Layer::Debris]);
        if *layers == player_mask {
            world.commands().entity(ctx.entity).insert(ColorMod::Enemy);
        } else {
            world
                .commands()
                .entity(ctx.entity)
                .insert(ColorMod::Friendly);
        }
    }
}

#[derive(Component)]
enum ColorMod {
    Friendly,
    Enemy,
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

/// Marks an enemy as a valid target for bullet collisions.
#[derive(Default, Component)]
#[require(CollidingEntities, ImageCollider, RigidBody::Kinematic, Sensor)]
pub struct Destructable;

fn handle_destructable_collision(
    bullets: Query<(Entity, &Damage, &BulletSprite, &GlobalTransform), With<Bullet>>,
    mut destructable: Query<(&CollidingEntities, Option<&mut Health>), With<Destructable>>,
    mut commands: Commands,
    mut writer: EventWriter<BulletCollisionEvent>,
) {
    let mut despawned = HashSet::new();
    for (colliding_entities, mut health) in destructable.iter_mut() {
        for (bullet, damage, sprite, transform) in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| bullets.get(entity))
        {
            if despawned.insert(bullet) {
                if let Some(health) = health.as_mut() {
                    health.damage(**damage);
                }

                writer.write(BulletCollisionEvent::new(
                    sprite.cell,
                    transform.compute_transform(),
                    BulletSource::Player,
                ));
                commands.entity(bullet).despawn();
            }
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
    camera: Single<Entity, With<OuterCamera>>,
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
            BulletSource::Player => {}
            BulletSource::Enemy => {
                let on_end = OnEnd::new(&mut commands, |mut commands: Commands| {
                    commands.remove_post_process::<GlitchSettings, OuterCamera>();
                    commands.remove_post_process::<GlitchIntensity, OuterCamera>();
                });

                commands.post_process::<OuterCamera>(GlitchSettings::default());
                commands.post_process::<OuterCamera>(GlitchIntensity::default());
                commands
                    .animation()
                    .insert_tween_here(
                        Duration::from_secs_f32(0.4),
                        EaseKind::Linear,
                        camera.into_target().with(glitch_intensity(0.3, 0.0)),
                    )
                    .insert(on_end);

                commands.add_trauma(0.15);
                commands.animation().insert_tween_here(
                    Duration::from_secs_f32(0.25),
                    EaseKind::Linear,
                    TargetResource.with(physics_time_mult(0.25, 1.)),
                );
            }
        }
    }
}
