use self::{
    emitter::{BULLET_DAMAGE, BULLET_SPEED},
    homing::HomingRotate,
};
use crate::{
    GameState, Layer,
    animation::{AnimationController, AnimationIndices, AnimationMode},
    assets::{self, MISC_PATH, MiscLayout},
    auto_collider::ImageCollider,
    bounds::WallDespawn,
    health::{Damage, Dead, Health},
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
                (handle_bullet_collision, despawn_dead_bullets)
                    .chain()
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
    pub fn target_layer(target: impl Into<LayerMask>) -> CollisionLayers {
        CollisionLayers::new(
            Layer::Bullet,
            [
                target.into(),
                [Layer::Bounds, Layer::Debris, Layer::Bullet].into(),
            ],
        )
    }

    fn add_color_mod(mut world: DeferredWorld, ctx: HookContext) {
        let layers = world.get::<CollisionLayers>(ctx.entity).unwrap();
        if layers.filters.has_all(Layer::Player) {
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
#[require(Bullet, ImageCollider, BulletSprite::from_cell(0, 1))]
pub struct BasicBullet;

#[derive(Clone, Copy, Component)]
#[require(Bullet, ImageCollider, BulletSprite::from_cell(5, 2))]
pub struct Missile;

#[derive(Clone, Copy, Component)]
#[require(Bullet, ImageCollider, BulletSprite::from_cell(4, 3))]
#[component(on_remove = Self::explode)]
pub struct Mine;

#[derive(Clone, Copy, Component)]
#[require(Bullet, ImageCollider, BulletSprite::from_cell(3, 1))]
pub struct Orb;

impl Mine {
    fn explode(mut world: DeferredWorld, ctx: HookContext) {
        let transform = *world.get::<Transform>(ctx.entity).unwrap();
        let mine_layers = *world.get::<CollisionLayers>(ctx.entity).unwrap();

        let layers = if mine_layers.filters.has_all(Layer::Player) {
            Bullet::target_layer(Layer::Player)
        } else {
            Bullet::target_layer(Layer::Enemy)
        };

        let dirs = [
            (Direction::NorthEast, -std::f32::consts::PI / 4.),
            (Direction::NorthWest, std::f32::consts::PI / 4.),
            (Direction::SouthEast, std::f32::consts::PI / 4.),
            (Direction::SouthWest, -std::f32::consts::PI / 4.),
            //
            (Direction::North, 0.),
            (Direction::South, 0.),
            (Direction::East, -std::f32::consts::PI / 2.),
            (Direction::West, std::f32::consts::PI / 2.),
        ];

        for (dir, rot) in dirs.into_iter() {
            world.commands().spawn((
                BasicBullet,
                LinearVelocity(BULLET_SPEED * dir.to_vec2() * Vec2::splat(0.4)),
                transform.with_rotation(Quat::from_rotation_z(rot)),
                layers,
                Damage::new(BULLET_DAMAGE),
            ));
        }
    }
}

/// Marks an enemy as a valid target for bullet collisions.
#[derive(Default, Component)]
#[require(CollidingEntities, ImageCollider, RigidBody::Kinematic, Sensor)]
pub struct Destructable;

fn handle_bullet_collision(
    bullets: Query<
        (
            Entity,
            &Damage,
            &BulletSprite,
            &GlobalTransform,
            &CollisionLayers,
        ),
        With<Bullet>,
    >,
    mut destructable: Query<(&CollidingEntities, Option<&mut Health>, Option<&Player>)>,
    mut commands: Commands,
    mut writer: EventWriter<BulletCollisionEvent>,
) {
    let mut despawned = HashSet::new();
    for (colliding_entities, mut health, player) in destructable.iter_mut() {
        for (bullet, damage, sprite, transform, layers) in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| bullets.get(entity))
        {
            if despawned.insert(bullet) {
                if let Some(health) = health.as_mut() {
                    health.damage(**damage);
                }

                let source = if layers.filters.has_all(Layer::Player) {
                    BulletSource::Enemy
                } else {
                    BulletSource::Player
                };

                writer.write(BulletCollisionEvent::new(
                    sprite.cell,
                    transform.compute_transform(),
                    source,
                    player.is_some(),
                ));
                commands.entity(bullet).despawn();
            }
        }
    }
}

fn despawn_dead_bullets(
    mut commands: Commands,
    bullets: Query<Entity, (With<Bullet>, With<Dead>)>,
) {
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Event)]
pub struct BulletCollisionEvent {
    /// The cell in the projectile spritesheet.
    pub cell: UVec2,
    pub transform: Transform,
    pub source: BulletSource,
    pub hit_player: bool,
}

impl BulletCollisionEvent {
    pub fn new(
        cell: UVec2,
        mut transform: Transform,
        source: BulletSource,
        hit_player: bool,
    ) -> Self {
        transform.translation = transform.translation.round();
        transform.translation.z = 1.;
        Self {
            cell,
            transform,
            source,
            hit_player,
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
                if event.hit_player {
                    commands.spawn((
                        SamplePlayer::new(server.load("audio/sfx/melee.wav")),
                        PitchRange(0.98..1.02),
                        PlaybackSettings {
                            volume: Volume::Linear(0.25),
                            ..PlaybackSettings::ONCE
                        },
                    ));

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
}
