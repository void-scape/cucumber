use self::{
    formation::{DEFAULT_FORMATION_VEL, FormationPlugin, FormationSet, Platoon},
    movement::*,
    timeline::LARGEST_SPRITE_SIZE,
};
use crate::{
    Avian, DespawnRestart, GameState, Layer,
    animation::AnimationSprite,
    assets,
    asteroids::SpawnCluster,
    auto_collider::ImageCollider,
    bounds::WallDespawn,
    bullet::{
        Destructable, Direction,
        emitter::{
            BuckShotEmitter, BulletModifiers, CrisscrossEmitter, MineEmitter, Rate,
            SpiralOrbEmitter, SwarmEmitter, TargetPlayer, WallEmitter,
        },
    },
    effects::Explosion,
    health::{Dead, Health},
    pickups::PowerUp,
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{GREEN, RED, WHITE, YELLOW},
    prelude::*,
    time::TimeSystem,
};
use bevy_enoki::prelude::*;
use bevy_optix::{debug::DebugRect, shake::TraumaCommands};
use bevy_tween::{
    interpolate::{rotation, sprite_color, translation},
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use rand::{Rng, seq::IteratorRandom};
use std::{f32::consts::PI, ops::Range, time::Duration};
use strum::IntoEnumIterator;

pub mod formation;
pub mod movement;
pub mod swarm;
pub mod timeline;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDeathEvent>()
            .add_plugins((FormationPlugin, MovementPlugin))
            .add_systems(OnEnter(GameState::Game), timeline::start_waves)
            .add_systems(Avian, swarm::swarm_movement)
            .add_systems(
                Update,
                (
                    timeline::update_waves.before(FormationSet),
                    insert_enemy_sprites.after(FormationSet),
                    (add_low_health_effects, death_effects),
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(
                PostUpdate,
                (handle_death, despawn_enemy).run_if(in_state(GameState::Game)),
            );

        #[cfg(debug_assertions)]
        app.add_systems(First, timeline::timeline_skip.after(TimeSystem));
    }
}

// #############
//    ENEMIES
// #############

#[derive(Default, Component)]
#[require(Transform, Visibility, Destructable, Trauma)]
pub struct Enemy;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(1.),
    EnemySprite8::cell(UVec2::new(4, 0)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    SwarmEmitter,
    BulletModifiers {
        rate: Rate::Factor(0.2),
        ..Default::default()
    },
    Trauma(0.04),
    Drops::new(0.2, 0.5),
    Explosion::Small,
)]
pub struct Swarm;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(20.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(4, 4)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    BuckShotEmitter,
    BulletModifiers {
        speed: 0.4,
        ..Default::default()
    },
    Drops::splat(8),
    DropPowerUp,
    Explosion::Big,
)]
pub struct BuckShot;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(20.),
    LowHealthEffects,
    DebugRect::from_size_color(Vec2::splat(12.), GREEN),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    WallEmitter,
    TargetPlayer,
    BulletModifiers {
        speed: 0.8,
        ..Default::default()
    },
    Drops::splat(8),
    Explosion::Big,
)]
pub struct WallShooter;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(10.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(4, 3)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    MineEmitter,
    Drops::splat(3),
    Explosion::Big,
)]
pub struct MineThrower;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(20.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(3, 4)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    SpiralOrbEmitter,
    Drops::splat(8),
    DropPowerUp,
    Explosion::Big,
)]
pub struct OrbSlinger;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(15.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(2, 4)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    CrisscrossEmitter,
    Drops::splat(6),
    Explosion::Big,
)]
pub struct CrissCross;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(8.),
    LowHealthEffects,
    DebugRect::from_size_color(Vec2::splat(8.), RED),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    Drops::splat(6),
    Explosion::Small,
)]
pub struct LaserNode;

#[derive(Default, Component)]
#[require(
    Transform,
    Visibility,
    DespawnRestart,
    ImageCollider,
    DebugRect::from_size_color(Vec2::splat(8.), YELLOW)
)]
pub struct InvincibleLaserNode;

// #############
//    Systems
// #############

#[derive(Component)]
#[require(Visibility)]
pub struct EnemySprite8 {
    path: &'static str,
    cell: UVec2,
}

impl EnemySprite8 {
    pub fn new(path: &'static str, cell: UVec2) -> Self {
        Self { path, cell }
    }

    pub fn cell(cell: UVec2) -> Self {
        Self::new(assets::SHIPS_PATH, cell)
    }
}

#[derive(Component)]
#[require(Visibility)]
struct EnemySprite16 {
    path: &'static str,
    cell: UVec2,
}

impl EnemySprite16 {
    pub fn new(path: &'static str, cell: UVec2) -> Self {
        Self { path, cell }
    }

    pub fn cell(cell: UVec2) -> Self {
        Self::new(assets::SHIPS_PATH, cell)
    }
}

fn insert_enemy_sprites(
    mut commands: Commands,
    server: Res<AssetServer>,
    sprite8: Query<(Entity, &EnemySprite8)>,
    sprite16: Query<(Entity, &EnemySprite16)>,
) {
    for (entity, sprite) in sprite8.iter() {
        commands
            .entity(entity)
            .insert(assets::sprite_rect8(&server, sprite.path, sprite.cell))
            .remove::<EnemySprite8>();
    }

    for (entity, sprite) in sprite16.iter() {
        commands
            .entity(entity)
            .insert(assets::sprite_rect16(&server, sprite.path, sprite.cell))
            .remove::<EnemySprite16>();
    }
}

#[derive(Component)]
struct Trauma(f32);

impl Default for Trauma {
    fn default() -> Self {
        Self(0.18)
    }
}

#[derive(Component)]
struct Drops {
    parts: DropCount,
    shield: DropCount,
}

impl Drops {
    pub fn new(parts: impl Into<DropCount>, shield: impl Into<DropCount>) -> Self {
        Self {
            parts: parts.into(),
            shield: shield.into(),
        }
    }

    pub fn splat(count: impl Into<DropCount>) -> Self {
        let count = count.into();
        Self {
            parts: count.clone(),
            shield: count,
        }
    }
}

#[derive(Clone)]
enum DropCount {
    Static(usize),
    Range(Range<usize>),
    One(f32),
}

impl DropCount {
    pub fn count(&self, rng: &mut impl Rng) -> usize {
        match self {
            DropCount::Static(c) => *c,
            DropCount::Range(r) => rng.random_range(r.clone()),
            DropCount::One(o) => rng.random_bool(*o as f64) as usize,
        }
    }
}

impl Into<DropCount> for usize {
    fn into(self) -> DropCount {
        DropCount::Static(self)
    }
}

impl Into<DropCount> for Range<usize> {
    fn into(self) -> DropCount {
        DropCount::Range(self)
    }
}

impl Into<DropCount> for f32 {
    fn into(self) -> DropCount {
        DropCount::One(self)
    }
}

#[derive(Default, Component)]
pub struct DropPowerUp;

#[derive(Event)]
pub struct EnemyDeathEvent {
    pub entity: Entity,
    pub position: Vec2,
    pub trauma: f32,
}

fn handle_death(
    q: Query<
        (
            Entity,
            &GlobalTransform,
            &Trauma,
            Option<&Drops>,
            Option<&DropPowerUp>,
            Option<&Explosion>,
        ),
        (With<Dead>, With<Enemy>),
    >,
    mut commands: Commands,
    mut deaths: EventWriter<EnemyDeathEvent>,
    mut clusters: EventWriter<SpawnCluster>,
) {
    let mut rng = rand::rng();
    for (entity, gt, trauma, drops, power_up, explosion) in q.iter() {
        if explosion.is_some_and(|e| *e == Explosion::Big) {
            let sign = if rng.random_bool(0.5) { -1. } else { 1. };
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(1.5),
                    EaseKind::QuadraticOut,
                    entity.into_target().with(translation(
                        gt.translation(),
                        gt.translation().with_z(-903.5),
                    )),
                )
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(1.5),
                    EaseKind::QuadraticIn,
                    entity.into_target().with(rotation(
                        gt.rotation(),
                        Quat::from_rotation_z(sign * PI / 6.) + Quat::from_rotation_x(PI / 2.),
                    )),
                )
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(1.5),
                    EaseKind::Linear,
                    entity
                        .into_target()
                        .with(sprite_color(WHITE.into(), Color::srgb(0.5, 0.5, 0.5))),
                )
                .remove::<(Enemy, BulletModifiers, Platoon, ChildOf, Dead, Explosion)>()
                .insert((
                    WallDespawn,
                    LinearVelocity(DEFAULT_FORMATION_VEL),
                    DespawnRestart,
                    CollisionLayers::new(Layer::Bullet, Layer::Bounds),
                ));
        } else {
            commands.entity(entity).despawn();
        }

        let position = gt.compute_transform().translation.xy();
        deaths.write(EnemyDeathEvent {
            entity,
            position,
            trauma: trauma.0,
        });

        if let Some(drops) = drops {
            clusters.write(SpawnCluster {
                parts: drops.parts.count(&mut rng),
                shield: drops.shield.count(&mut rng),
                position,
            });
        }

        if power_up.is_some() {
            commands.spawn((PowerUp, gt.compute_transform()));
        }
    }
}

fn death_effects(mut commands: Commands, mut reader: EventReader<EnemyDeathEvent>) {
    for event in reader.read() {
        commands.add_trauma(event.trauma);
    }
}

#[derive(Default, Component)]
struct LowHealthEffects;

#[derive(Component)]
struct AppliedLowHealthEffects;

fn add_low_health_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    query: Query<(Entity, &Health), (With<LowHealthEffects>, Without<AppliedLowHealthEffects>)>,
) {
    const DIST: f32 = 8.;
    const Y_OFFSET: f32 = 5.;

    let mut rng = rand::rng();
    for (entity, health) in query.iter() {
        if health.current() <= health.max() / 2.0 {
            let mut chosen = [Direction::default(); 3];
            Direction::iter().choose_multiple_fill(&mut rng, &mut chosen);
            commands
                .entity(entity)
                .insert(AppliedLowHealthEffects)
                .with_children(|root| {
                    for dir in chosen.iter() {
                        root.spawn((
                            Transform::from_scale(Vec3::splat(0.2)).with_translation(
                                ((dir.to_vec2() * DIST) + Vec2::Y * Y_OFFSET).extend(1.),
                            ),
                            AnimationSprite::repeating(
                                "fire_sparks.png",
                                rng.random_range(0.025..0.05),
                                0..=18,
                            ),
                        ));
                    }
                    root.spawn((
                        ParticleSpawner::default(),
                        ParticleEffectHandle(server.load("particles/ship_fire.ron")),
                        Transform::from_translation(Vec2::ZERO.extend(-100.)),
                    ));
                });
        }
    }
}

fn despawn_enemy(mut commands: Commands, enemies: Query<(Entity, &GlobalTransform), With<Enemy>>) {
    for (entity, gt) in enemies.iter() {
        if gt.translation().y < -crate::HEIGHT / 2. - LARGEST_SPRITE_SIZE {
            commands.entity(entity).despawn();
        }
    }
}
