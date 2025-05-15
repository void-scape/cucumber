use self::{
    emitter::{BULLET_DAMAGE, BULLET_SPEED, MINE_HEALTH, MISSILE_HEALTH},
    homing::HomingRotate,
};
use crate::{
    DespawnRestart, Layer,
    animation::AnimationSprite,
    assets::{self, MISC_PATH},
    auto_collider::ImageCollider,
    bounds::WallDespawn,
    health::{Damage, DamageEvent, Dead, Health},
    player::Player,
    points::PointEvent,
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{MAGENTA, RED, SKY_BLUE},
    ecs::{component::HookContext, world::DeferredWorld},
    platform::collections::HashSet,
    prelude::*,
};
use bevy_seedling::{
    prelude::Volume,
    sample::{PlaybackSettings, SamplePlayer},
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
                init_bullet_sprite
                    .in_set(BulletSystems::Sprite)
                    .after(BulletSystems::Collision),
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
    pub fn to_vec2(self) -> Vec2 {
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
            ColorMod::Purple => {
                sprite.color = MAGENTA.into();
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
pub struct MaxLifetime(pub Duration);

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
#[require(
    Polarity,
    Sensor,
    RigidBody::Kinematic,
    WallDespawn,
    DespawnRestart,
    CollisionLayers = Self::target_layer(Layer::Player)
)]
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
        if !world.get::<ColorMod>(ctx.entity).is_none() {
            return;
        }

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
    Purple,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct PlayerBullet;

#[derive(Clone, Copy, Component)]
#[require(Bullet, ImageCollider, BulletSprite::from_cell(1, 2))]
pub struct BasicBullet;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    HomingRotate,
    Collider::rectangle(2., 2.),
    BulletSprite::from_cell(2, 7)
)]
pub struct Arrow;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    Destructable,
    Health::full(MISSILE_HEALTH),
    Collider::rectangle(2., 2.),
    BulletSprite::from_cell(5, 2)
)]
pub struct Missile;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    Destructable,
    Health::full(MINE_HEALTH),
    Collider::circle(1.5),
    BulletSprite::from_cell(4, 3)
)]
pub struct Mine;

#[derive(Clone, Copy, Component)]
#[require(Bullet, Collider::circle(1.5), BulletSprite::from_cell(3, 1))]
pub struct Orb;

/// Marks an enemy as a valid target for bullet collisions.
#[derive(Default, Component)]
#[require(CollidingEntities, RigidBody::Kinematic, Sensor)]
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
    destructable: Query<
        (
            Entity,
            &CollidingEntities,
            &CollisionLayers,
            Option<&Player>,
            Option<&Bullet>,
        ),
        With<Health>,
    >,
    mut commands: Commands,
    mut writer: EventWriter<BulletCollisionEvent>,
    mut damage_writer: EventWriter<DamageEvent>,
) {
    let mut despawned = HashSet::new();
    for (entity, colliding_entities, destructable_layers, player, destructable_bullet) in
        destructable.iter()
    {
        for (bullet, damage, sprite, transform, layers) in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| bullets.get(entity))
            .filter(|(_, _, _, _, layers)| {
                if destructable_bullet.is_some() {
                    destructable_layers.filters.has_all(Layer::Player)
                        && layers.filters.has_all(Layer::Enemy)
                } else {
                    true
                }
            })
        {
            if despawned.insert(bullet) {
                damage_writer.write(DamageEvent {
                    damage: damage.damage(),
                    entity,
                });

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
    server: Res<AssetServer>,
    bullets: Query<Entity, (With<Dead>, With<Bullet>, Without<Mine>)>,
    mines: Query<(Entity, &Transform), (With<Dead>, With<Mine>)>,
    mut writer: EventWriter<PointEvent>,
) {
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
    }

    for (entity, transform) in mines.iter() {
        commands.entity(entity).despawn();
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
            commands.spawn((
                BasicBullet,
                LinearVelocity(BULLET_SPEED * dir.to_vec2() * Vec2::splat(0.4)),
                transform.with_rotation(Quat::from_rotation_z(rot)),
                Damage::new(BULLET_DAMAGE),
            ));
        }

        commands.spawn((
            Transform::from_translation(transform.translation.xy().extend(-99.))
                .with_scale(Vec3::splat(0.5)),
            AnimationSprite::once("explosion3.png", 0.05, 0..=11),
        ));

        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/explosion3.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));

        writer.write(PointEvent {
            points: 10,
            position: transform.translation.xy(),
        });
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

fn bullet_collision_effects(mut commands: Commands, mut reader: EventReader<BulletCollisionEvent>) {
    for event in reader.read() {
        commands.spawn((
            event.transform,
            AnimationSprite::once(MISC_PATH, 0.05, 83..=86),
        ));
    }
}
