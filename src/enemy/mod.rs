use self::{
    formation::{FormationPlugin, FormationSet},
    movement::*,
};
use crate::{
    Avian, GameState, Layer,
    animation::AnimationSprite,
    assets,
    asteroids::SpawnCluster,
    atlas_layout,
    auto_collider::ImageCollider,
    bounds::WallDespawn,
    bullet::{
        Destructable, Direction,
        emitter::{
            BuckShotEmitter, BulletModifiers, CrisscrossEmitter, MineEmitter, Rate,
            SpiralOrbEmitter, SwarmEmitter,
        },
    },
    effects::{Size, SpawnExplosion},
    health::{Dead, Health},
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLACK, WHITE},
    prelude::*,
    time::TimeSystem,
};
use bevy_optix::shake::TraumaCommands;
use bevy_seedling::{
    prelude::Volume,
    sample::{PlaybackSettings, SamplePlayer},
};
use bevy_tween::{
    combinator::tween,
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
            .add_systems(
                Startup,
                (
                    init_explosion_layout,
                    init_cruiser_explosion_layout,
                    init_explosion1_layout,
                ),
            )
            .add_systems(OnEnter(GameState::Game), timeline::start_waves)
            .add_systems(Avian, swarm::swarm_movement)
            .add_systems(
                Update,
                (
                    timeline::update_waves.before(FormationSet),
                    insert_enemy_sprites.after(FormationSet),
                    (add_low_health_effects, death_effects, handle_death),
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );

        #[cfg(debug_assertions)]
        app.add_systems(First, timeline::timeline_skip.after(TimeSystem));
    }
}

atlas_layout!(ExplosionLayout, init_explosion_layout, 64, 10, 1);
atlas_layout!(CruiserExplosion, init_cruiser_explosion_layout, 128, 14, 1);
atlas_layout!(Explosion1Layout, init_explosion1_layout, 64, 8, 9);

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
    LowHealthEffects,
    EnemySprite8::cell(UVec2::new(4, 0)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
    swarm::SwarmMovement,
    SwarmEmitter,
    BulletModifiers {
        rate: Rate::Factor(0.2),
        ..Default::default()
    },
    Trauma(0.04),
    Drops::new(0.2, 0.5),
)]
pub struct Swarm;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(15.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(4, 4)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
    BuckShotEmitter,
    BulletModifiers {
        speed: 0.4,
        ..Default::default()
    },
    Drops::splat(8),
)]
pub struct BuckShot;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(10.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(4, 3)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
    MineEmitter,
    Drops::splat(3),
)]
pub struct MineThrower;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(20.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(3, 4)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
    SpiralOrbEmitter,
    Drops::splat(8),
)]
pub struct OrbSlinger;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(15.),
    LowHealthEffects,
    EnemySprite16::cell(UVec2::new(2, 4)),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
    CrisscrossEmitter,
    Drops::splat(6),
)]
pub struct CrissCross;

// #############
//    Systems
// #############

#[derive(Component)]
#[require(Visibility)]
struct EnemySprite8 {
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

//fn configure(&self, commands: &mut EntityCommands) {
//    let mut rng = rand::rng();
//    match self {
//        Self::Gunner => commands.with_child((
//            BulletModifiers {
//                rate: Rate::Factor(0.75),
//                ..Default::default()
//            },
//            SoloEmitter::player(),
//        )),
//        Self::Missile => commands
//            .insert((
//                BackAndForth {
//                    radius: rng.random_range(9.0..11.0),
//                    speed: rng.random_range(2.2..3.4),
//                },
//                Angle(rng.random_range(0.0..2.0)),
//            ))
//            .with_child((
//                HomingEmitter::<Player>::player(),
//                TurnSpeed(55.),
//                BulletModifiers {
//                    rate: Rate::Factor(0.25),
//                    speed: 0.75,
//                    ..Default::default()
//                },
//                MaxLifetime(Duration::from_secs_f32(5.0)),
//            )),
//        Self::MineThrower => commands
//            .insert((
//                BackAndForth {
//                    radius: rng.random_range(4.0..6.0),
//                    speed: rng.random_range(1.2..2.4),
//                },
//                Angle(rng.random_range(0.0..2.0)),
//            ))
//            .with_child(MineEmitter::player()),
//        Self::OrbSlinger => commands.with_child(OrbEmitter::player()),
//        Self::BuckShot => commands.with_child((
//            BuckShotEmitter::player().with_all(4, 1.5, 0.2),
//            BulletModifiers {
//                speed: 0.4,
//                ..Default::default()
//            },
//        )),
//        Self::CrissCross => commands.with_child(CrisscrossEmitter::player()),
//        Self::Swarm => commands.insert(swarm::SwarmMovement),
//    };
//}

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
            &Health,
            &GlobalTransform,
            &Transform,
            &Trauma,
            Option<&Drops>,
        ),
        (With<Dead>, With<Enemy>),
    >,
    mut commands: Commands,
    mut deaths: EventWriter<EnemyDeathEvent>,
    mut clusters: EventWriter<SpawnCluster>,
) {
    let mut rng = rand::rng();
    for (entity, health, gt, transform, trauma, drops) in q.iter() {
        if health.max() > 1. {
            let sign = if rng.random_bool(0.5) { -1. } else { 1. };
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(1.5),
                    EaseKind::QuadraticOut,
                    entity.into_target().with(translation(
                        transform.translation,
                        transform.translation.with_z(-1.),
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
                        .with(sprite_color(WHITE.into(), Color::srgb(0.2, 0.2, 0.2))),
                )
                .remove::<(Enemy, BulletModifiers, LinearVelocity)>()
                .insert((
                    WallDespawn,
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
    }
}

fn death_effects(
    mut commands: Commands,
    mut reader: EventReader<EnemyDeathEvent>,
    mut writer: EventWriter<SpawnExplosion>,
) {
    for event in reader.read() {
        commands.add_trauma(event.trauma);
        writer.write(SpawnExplosion {
            position: event.position,
            size: if event.trauma == Trauma::default().0 {
                Size::Big
            } else {
                Size::Small
            },
        });
    }
}

#[derive(Default, Component)]
struct LowHealthEffects;

#[derive(Component)]
struct AppliedLowHealthEffects;

fn add_low_health_effects(
    mut commands: Commands,
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
                        Transform::from_scale(Vec3::splat(0.2))
                            .with_translation(Vec2::ZERO.extend(-1.)),
                        AnimationSprite::repeating(
                            "sparks.png",
                            rng.random_range(0.025..0.0251),
                            0..=19,
                        ),
                    ));
                });
        }
    }
}
