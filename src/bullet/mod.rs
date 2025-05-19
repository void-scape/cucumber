use self::{
    emitter::{BULLET_DAMAGE, MINE_HEALTH, MISSILE_HEALTH, PLAYER_BULLET_SPEED},
    homing::HomingRotate,
};
use crate::{
    DespawnRestart, Layer,
    animation::{AnimationAppExt, AnimationSprite},
    assets::{self, MISC_PATH},
    auto_collider::ImageCollider,
    bounds::WallDespawn,
    effects::{AlwaysBlast, Blasters, Explosion, SpawnExplosion},
    health::{Damage, DamageEvent, Dead, Health},
    player::Player,
    points::PointEvent,
    sprites::{self, CellSize},
    tween::OnEnd,
};
use avian2d::{math::FRAC_PI_2, prelude::*};
use bevy::{
    color::palettes::css::{MAGENTA, RED, SKY_BLUE},
    platform::collections::HashSet,
    prelude::*,
    sprite::Anchor,
};
use bevy_enoki::{ParticleEffectHandle, ParticleSpawner};
use bevy_tween::{
    interpolate::sprite_color,
    prelude::{AnimationBuilderExt, EaseKind, Repeat, RepeatStyle},
    tween::IntoTarget,
};
use rand::Rng;
use std::{f32::consts::PI, time::Duration};
use strum_macros::EnumIter;

pub mod emitter;
pub mod homing;
pub mod player;

const GRAZE_DIST: f32 = 15.;
const GRAZE_POINTS: usize = 5;

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
                (grazing, handle_bullet_collision, despawn_dead_bullets)
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
            )
            .register_layout(
                "orb.png",
                TextureAtlasLayout::from_grid(UVec2::splat(8), 3, 1, None, None),
            )
            .register_layout(
                "orb1.png",
                TextureAtlasLayout::from_grid(UVec2::splat(8), 3, 1, None, None),
            )
            .register_layout(
                "bomb.png",
                TextureAtlasLayout::from_grid(UVec2::splat(8), 8, 1, None, None),
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
            Option<&ColorMod>,
            Option<&LinearVelocity>,
            Option<&HomingRotate>,
        ),
        Without<Sprite>,
    >,
    server: Res<AssetServer>,
) {
    for (entity, sprite, color, velocity, rotation) in bullets.iter() {
        let brightness = sprite.brightness;
        let alpha = sprite.alpha;

        let mut sprite = sprites::sprite_rect(&server, sprite.path, CellSize::Eight, sprite.cell);
        if rotation.is_none() {
            if let Some(velocity) = velocity {
                sprite.flip_y = velocity.0.y < 0.;
                sprite.flip_x = velocity.0.x < 0.;
            }
        }

        if let Some(color) = color {
            sprite.color = match color {
                ColorMod::Red => RED,
                ColorMod::Blue => SKY_BLUE,
                ColorMod::Purple => MAGENTA,
            }
            .with_luminance(brightness)
            .with_alpha(alpha)
            .into();
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
        Self::new(10.)
    }
}

impl Lifetime {
    fn new(secs: f32) -> Self {
        Self(Timer::new(Duration::from_secs_f32(secs), TimerMode::Once))
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
}

#[derive(Clone, Copy, EnumIter, Component)]
enum ColorMod {
    Blue,
    Red,
    Purple,
}

#[derive(Component)]
pub struct BulletSprite {
    path: &'static str,
    cell: UVec2,
    brightness: f32,
    alpha: f32,
}

impl BulletSprite {
    pub fn from_cell(x: u32, y: u32) -> Self {
        Self {
            path: assets::PROJECTILES_COLORED_PATH,
            cell: UVec2::new(x, y),
            brightness: 1.,
            alpha: 1.,
        }
    }

    pub fn with_brightness(mut self, brightness: f32) -> Self {
        self.brightness = brightness;
        self
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
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
    BulletSprite::from_cell(5, 5)
)]
pub struct Missile;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    Collider::rectangle(2., 2.),
    BulletSprite::from_cell(5, 2),
    Blasters(const { &[Vec3::new(0., -5., -1.)] }),
    AlwaysBlast,
)]
pub struct Rocket;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    Destructable,
    Health::full(MINE_HEALTH),
    Collider::circle(1.5),
    AnimationSprite::repeating("bomb.png", 0.1, 0..8),
)]
pub struct Mine;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    Collider::circle(1.5),
    AnimationSprite::repeating("orb.png", 0.2, 0..3),
    AngularVelocity(FRAC_PI_2),
)]
pub struct RedOrb;

#[derive(Clone, Copy, Component)]
#[require(
    Bullet,
    Collider::circle(1.5),
    AnimationSprite::repeating("orb1.png", 0.2, 0..3),
    AngularVelocity(FRAC_PI_2),
)]
pub struct BlueOrb;

/// Marks an enemy as a valid target for bullet collisions.
#[derive(Default, Component)]
#[require(CollidingEntities, RigidBody::Kinematic, Sensor)]
pub struct Destructable;

#[derive(Component)]
struct Grazed;

fn grazing(
    mut commands: Commands,
    mut writer: EventWriter<PointEvent>,
    player: Single<&Transform, With<Player>>,
    bullets: Query<(Entity, &Transform), (With<Bullet>, Without<PlayerBullet>, Without<Grazed>)>,
) {
    let pp = player.translation.xy();
    for (entity, transform) in bullets.iter() {
        let position = transform.translation.xy();
        if position.distance(pp) < GRAZE_DIST {
            commands.entity(entity).insert(Grazed);
            writer.write(PointEvent {
                points: GRAZE_POINTS,
                position,
            });
        }
    }
}

fn handle_bullet_collision(
    bullets: Query<(Entity, &Damage, &GlobalTransform, &CollisionLayers), With<Bullet>>,
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
        for (bullet, damage, transform, layers) in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| bullets.get(entity))
            .filter(|(_, _, _, layers)| {
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
    mut explosions: EventWriter<SpawnExplosion>,
) {
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
    }

    let mut rng = rand::rng();
    for (entity, transform) in mines.iter() {
        commands.entity(entity).despawn();
        let dirs = [
            (Direction::NorthEast, -std::f32::consts::PI / 4.),
            (Direction::NorthWest, std::f32::consts::PI / 4.),
            (Direction::SouthEast, -3. * std::f32::consts::PI / 4.),
            (Direction::SouthWest, 3. * std::f32::consts::PI / 4.),
            //
            (Direction::North, 0.),
            (Direction::South, -std::f32::consts::PI),
            (Direction::East, -std::f32::consts::PI / 2.),
            (Direction::West, std::f32::consts::PI / 2.),
        ];

        let offset = rng.random_range(-PI..PI);
        for (dir, rot) in dirs.into_iter() {
            let rot = rot + offset;

            let mut sprite = Sprite::from_image(server.load("line.png"));
            sprite.anchor = Anchor::BottomCenter;
            let indicators = commands
                .spawn((
                    sprite,
                    transform
                        .with_scale(Vec3::new(1., crate::HEIGHT, 1.))
                        .with_rotation(Quat::from_rotation_z(rot)),
                ))
                .id();

            let t = *transform;
            let end = OnEnd::new(
                &mut commands,
                move |mut commands: Commands, server: Res<AssetServer>| {
                    commands.spawn((
                        Rocket,
                        ParticleSpawner::default(),
                        ParticleEffectHandle(server.load("particles/rocket.ron")),
                        HomingRotate,
                        LinearVelocity(
                            PLAYER_BULLET_SPEED
                                * (dir.to_vec2().rotate(Vec2::from_angle(offset)))
                                * 0.8,
                        ),
                        t.with_rotation(Quat::from_rotation_z(rot)),
                        Damage::new(BULLET_DAMAGE),
                    ));
                },
            );
            commands
                .entity(indicators)
                .animation()
                .repeat(Repeat::times(2))
                .repeat_style(RepeatStyle::PingPong)
                .insert_tween_here(
                    Duration::from_secs_f32(0.2),
                    EaseKind::BounceInOut,
                    indicators
                        .into_target()
                        .with(sprite_color(Color::WHITE, Color::WHITE.with_alpha(0.0))),
                )
                .insert(end);
        }

        explosions.write(SpawnExplosion {
            position: transform.translation.xy(),
            explosion: Explosion::Small,
        });

        writer.write(PointEvent {
            points: 10,
            position: transform.translation.xy(),
        });
    }
}

#[derive(Event)]
pub struct BulletCollisionEvent {
    pub transform: Transform,
    pub source: BulletSource,
    pub hit_player: bool,
}

impl BulletCollisionEvent {
    pub fn new(mut transform: Transform, source: BulletSource, hit_player: bool) -> Self {
        transform.translation = transform.translation.round();
        transform.translation.z = 1.;
        Self {
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
