use self::{
    formation::{DEFAULT_FORMATION_VEL, FormationPlugin, FormationSet, Platoon},
    movement::*,
    timeline::{ENEMY_Z, LARGEST_SPRITE_SIZE},
};
use crate::{
    DespawnRestart, GameState, Layer, assets,
    asteroids::SpawnCluster,
    auto_collider::ImageCollider,
    background::LAYER2,
    bullet::{
        Destructable, Direction,
        emitter::{
            BulletModifiers, ConvergentEmitter, CrisscrossEmitter, MineEmitter, SpiralOrbEmitter,
        },
    },
    effects::Explosion,
    health::{Dead, Health},
    pickups::PowerUp,
    player::Player,
    sprites::{
        BehaviorNodes, BehaviorRoot, CellSize, CellSprite, MultiSprite, SpriteBehavior,
        SpriteBundle,
    },
    tween::DespawnTweenFinish,
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{RED, YELLOW},
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
    time::TimeSystem,
};
use bevy_enoki::prelude::*;
use bevy_optix::{debug::DebugRect, shake::TraumaCommands};
use bevy_tween::{
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use rand::{Rng, seq::IteratorRandom};
use std::{f32::consts::PI, ops::Range, time::Duration};
use strum::IntoEnumIterator;

pub mod buckshot;
pub mod formation;
pub mod movement;
pub mod scout;
pub mod swarm;
pub mod timeline;
pub mod waller;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDeathEvent>()
            .add_plugins((FormationPlugin, MovementPlugin))
            .add_systems(OnEnter(GameState::Game), timeline::start_waves)
            .add_systems(
                Update,
                (
                    timeline::update_waves.before(FormationSet),
                    (add_low_health_effects, death_effects),
                    (face_player, face_velocity),
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
#[require(
    Transform,
    Visibility,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    Destructable,
    Trauma,
)]
pub struct Enemy;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    CellSprite::new24("ships.png", UVec2::new(2, 1)),
    Health::full(10.),
    LowHealthEffects,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    MineEmitter,
    Drops::splat(3),
    Explosion::Big,
)]
pub struct MineThrower;

#[derive(Default, Component)]
#[require(
    Enemy,
    Health::full(20.),
    Collider::circle(6.),
    CellSprite::new24("ships.png", UVec2::new(0, 1)),
    LowHealthEffects,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    SpiralOrbEmitter,
    Drops::splat(8),
    Explosion::Big,
)]
#[component(on_add = Self::sprites)]
pub struct OrbSlinger;

impl OrbSlinger {
    fn sprites(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).with_child((
            CellSprite::new24("ships.png", UVec2::new(0, 2)),
            RigidBody::Kinematic,
            AngularVelocity(0.4),
            // no behavior, just need to despawn on death
            BehaviorRoot(ctx.entity),
        ));
    }
}

#[derive(Default, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    SpriteBundle = Self::sprites(),
    Health::full(15.),
    LowHealthEffects,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    CrisscrossEmitter,
    Drops::splat(6),
    Explosion::Big,
)]
pub struct CrissCross;

impl CrissCross {
    fn sprites() -> SpriteBundle {
        SpriteBundle::new([
            MultiSprite::Static(CellSprite {
                path: "ships.png",
                size: CellSize::TwentyFour,
                cell: UVec2::new(4, 1),
                z: 0.,
            }),
            MultiSprite::Dynamic {
                sprite: CellSprite {
                    path: "ships.png",
                    size: CellSize::TwentyFour,
                    cell: UVec2::new(4, 2),
                    z: ENEMY_Z - 1.,
                },
                behavior: SpriteBehavior::Crisscross,
                position: Vec2::ZERO,
            },
        ])
    }
}

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

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::rectangle(12., 12.),
    CellSprite::new24("ships.png", UVec2::new(0, 1)),
    Health::full(10.),
    LowHealthEffects,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    ConvergentEmitter,
    Drops::splat(3),
    Explosion::Big,
)]
#[component(on_add = OrbSlinger::sprites)]
pub struct Convergence;

// #############
//    Systems
// #############

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

#[derive(Component)]
struct Trauma(f32);

impl Default for Trauma {
    fn default() -> Self {
        Self(0.18)
    }
}

impl Trauma {
    pub const NONE: Self = Trauma(0.);
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
            commands
                .entity(entity)
                .despawn_related::<BehaviorNodes>()
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(0.5),
                    EaseKind::Linear,
                    entity.into_target().with(translation(
                        gt.translation(),
                        gt.translation()
                            .with_z(LAYER2 - 10.)
                            .with_y(gt.translation().y + DEFAULT_FORMATION_VEL.y * 1.5),
                    )),
                )
                .remove::<(
                    Enemy,
                    BulletModifiers,
                    Platoon,
                    ChildOf,
                    Dead,
                    Explosion,
                    FacePlayer,
                )>()
                .insert((
                    DespawnTweenFinish,
                    //WallDespawn,
                    //LinearVelocity(DEFAULT_FORMATION_VEL),
                    //DespawnRestart,
                    //CollisionLayers::new(Layer::Bullet, Layer::Bounds),
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
    const DIST: f32 = 4.;
    const Y_OFFSET: f32 = 1.;

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
                                ((dir.to_vec2() * DIST) + Vec2::Y * Y_OFFSET).extend(0.1),
                            ),
                            ParticleSpawner::default(),
                            ParticleEffectHandle(server.load("particles/fire.ron")),
                        ));
                    }
                    root.spawn((
                        ParticleSpawner::default(),
                        ParticleEffectHandle(server.load("particles/ship_fire.ron")),
                        Transform::from_translation(Vec2::ZERO.extend(-0.1)),
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

#[derive(Default, Component)]
#[require(AngularVelocity)]
struct FacePlayer;

fn face_player(
    player: Single<&Transform, With<Player>>,
    time: Res<Time>,
    mut entities: Query<
        (&GlobalTransform, &mut AngularVelocity),
        (With<FacePlayer>, Without<Player>),
    >,
) {
    let pp = player.translation.xy();
    for (gt, mut ang_vel) in entities.iter_mut() {
        let p = gt.translation().xy();
        if p != Vec2::ZERO && pp != Vec2::ZERO {
            let dir_to_player = pp - p;
            let forward = gt.rotation().mul_vec3(Vec3::X).xy().normalize();

            let angle = forward.x.atan2(forward.y) - dir_to_player.x.atan2(dir_to_player.y);
            // Add 90 degrees (π/2 radians) rotation offset
            let angle_with_offset = angle + std::f32::consts::FRAC_PI_2;

            // Normalize the angle to be between -π and π
            let normalized_angle = (angle_with_offset + std::f32::consts::PI)
                % (2.0 * std::f32::consts::PI)
                - std::f32::consts::PI;
            ang_vel.0 = normalized_angle * 100. * time.delta_secs();
        } else {
            ang_vel.0 = 0.;
        }
    }
}

#[derive(Default, Component)]
#[require(AngularVelocity)]
struct FaceVelocity;

fn face_velocity(
    mut entities: Query<
        (&mut Rotation, &LinearVelocity),
        (With<FaceVelocity>, Changed<LinearVelocity>),
    >,
) {
    for (mut rotation, velocity) in entities.iter_mut() {
        if velocity.0 == Vec2::ZERO {
            *rotation = Rotation::radians(Vec2::NEG_Y.to_angle() + PI / 2.);
        } else {
            *rotation = Rotation::radians(velocity.to_angle() + PI / 2.);
        }
    }
}
