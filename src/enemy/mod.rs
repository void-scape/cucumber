use self::{
    formation::{
        Formation, FormationPlugin, FormationSet, double_buck_shot, double_crisscross,
        double_orb_slinger, quad_mine_thrower,
    },
    movement::*,
};
use crate::{
    Avian, GameState, Layer,
    animation::{AnimationController, AnimationIndices},
    assets,
    asteroids::{AsteroidSpawner, SpawnCluster},
    atlas_layout, boss,
    bullet::{
        Destructable, Direction, MaxLifetime,
        emitter::{
            BuckShotEmitter, BulletModifiers, CrisscrossEmitter, HomingEmitter, MineEmitter,
            OrbEmitter, ProximityEmitter, Rate, SoloEmitter,
        },
        homing::TurnSpeed,
    },
    health::{Dead, Health},
    player::Player,
};
use avian2d::prelude::*;
use bevy::{ecs::system::RunSystemOnce, prelude::*, time::TimeSystem};
use bevy_optix::shake::TraumaCommands;
use bevy_seedling::{
    prelude::Volume,
    sample::{PlaybackSettings, SamplePlayer},
};
use bevy_sequence::combinators::delay::run_after;
use formation::{crisscross, mine_thrower, orb_slinger, row, swarm};
use rand::{Rng, rngs::ThreadRng, seq::IteratorRandom};
use std::time::Duration;
use strum::IntoEnumIterator;

pub mod formation;
pub mod movement;
pub mod swarm;

#[cfg(not(debug_assertions))]
const START_DELAY: f32 = 1.5;
#[cfg(debug_assertions)]
const START_DELAY: f32 = 0.;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDeathEvent>()
            .add_plugins((FormationPlugin, MovementPlugin))
            .add_systems(
                Startup,
                (
                    init_fire_layout,
                    init_sparks_layout,
                    init_explosion_layout,
                    init_cruiser_explosion_layout,
                ),
            )
            .add_systems(OnEnter(GameState::Game), start_waves)
            .add_systems(Avian, swarm::swarm_movement)
            .add_systems(
                Update,
                (
                    update_waves.before(FormationSet),
                    (add_low_health_effects, death_effects, handle_death),
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );

        #[cfg(debug_assertions)]
        app.add_systems(First, timeline_skip.after(TimeSystem));
    }
}

atlas_layout!(FireLayout, init_fire_layout, 96, 4, 5);
atlas_layout!(SparksLayout, init_sparks_layout, 150, 5, 6);
atlas_layout!(ExplosionLayout, init_explosion_layout, 64, 10, 1);
atlas_layout!(CruiserExplosion, init_cruiser_explosion_layout, 128, 14, 1);

fn start_waves(mut commands: Commands) {
    if crate::SKIP_WAVES {
        commands.insert_resource(WaveTimeline::new(&[]));
    } else {
        commands.insert_resource(
            WaveTimeline::new_delayed(
                START_DELAY,
                &[
                    (swarm(), 8.),
                    (swarm(), 12.),
                    (double_buck_shot(), 14.),
                    //(row(), 2.),
                    (quad_mine_thrower(), 16.),
                    (swarm(), 8.),
                    (double_crisscross(), 2.),
                    (orb_slinger(), 16.),
                    (crisscross(), 2.),
                    (double_orb_slinger(), 10.),
                    (swarm(), 16.),
                ],
            ), //.skip(32.),
        );
    }

    //#[cfg(debug_assertions)]
    //commands.queue(|world: &mut World| {
    //    loop {
    //        if !world.resource::<WaveTimeline>().is_skipping() {
    //            break;
    //        }
    //        world.run_schedule(First);
    //        world.run_schedule(PreUpdate);
    //        world.run_schedule(RunFixedMainLoop);
    //        world.run_schedule(Update);
    //        world.run_schedule(PostUpdate);
    //        world.run_schedule(Last);
    //    }
    //});
}

#[derive(Resource)]
pub struct WaveTimeline {
    seq: Vec<(Formation, f32)>,
    timer: Timer,
    index: usize,
    finished: bool,
    skip: Option<Timer>,
}

impl WaveTimeline {
    pub fn new(seq: &[(Formation, f32)]) -> Self {
        Self::new_delayed(0., seq)
    }

    pub fn new_delayed(delay: f32, seq: &[(Formation, f32)]) -> Self {
        Self {
            seq: seq.to_vec(),
            timer: Timer::from_seconds(delay, TimerMode::Repeating),
            index: 0,
            finished: false,
            skip: None,
        }
    }

    pub fn skip(mut self, secs: f32) -> Self {
        self.skip = Some(Timer::from_seconds(secs, TimerMode::Once));
        self
    }

    pub fn is_skipping(&self) -> bool {
        self.skip.is_some()
    }

    pub fn tick(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }

    pub fn next(&mut self) -> Option<Formation> {
        if self.timer.just_finished() {
            match self.seq.get(self.index) {
                Some((formation, duration)) => {
                    self.timer.set_duration(Duration::from_secs_f32(*duration));
                    self.index += 1;
                    Some(formation.clone())
                }
                None => {
                    self.finished = true;
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn finished(&self) -> bool {
        self.finished
    }
}

#[cfg(debug_assertions)]
fn timeline_skip(
    mut commands: Commands,
    controller: Option<ResMut<WaveTimeline>>,
    mut time: ResMut<Time<Virtual>>,
    player: Single<Entity, With<Player>>,
) {
    let Some(mut controller) = controller else {
        return;
    };

    if controller.is_added() && controller.skip.is_some() {
        commands.entity(*player).insert(ColliderDisabled);
    }

    let Some(timer) = controller.skip.as_mut() else {
        return;
    };

    time.advance_by(Duration::from_millis(16));
    timer.tick(time.delta());
    if timer.finished() {
        controller.skip = None;
        commands.entity(*player).remove::<ColliderDisabled>();
    }
}

fn update_waves(
    mut commands: Commands,
    controller: Option<ResMut<WaveTimeline>>,
    formations: Query<&Formation>,
    time: Res<Time>,
    mut asteroids: ResMut<AsteroidSpawner>,
) {
    let Some(mut controller) = controller else {
        return;
    };

    controller.tick(&time);
    if let Some(formation) = controller.next() {
        commands.spawn(formation);
    }

    if controller.finished() && formations.is_empty() {
        asteroids.0 = false;
        commands.remove_resource::<WaveTimeline>();

        if crate::SKIP_WAVES {
            commands.queue(|world: &mut World| world.run_system_once(boss::gradius));
        } else {
            info!("ran out of formations, spawning boss");
            run_after(Duration::from_secs_f32(5.), boss::gradius, &mut commands);
        }
    }
}

#[derive(Default, Component)]
pub struct Enemy;

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, Hash)]
#[require(
    Transform,
    Visibility,
    LinearVelocity,
    Destructable,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
    Enemy,
)]
pub enum EnemyType {
    Gunner,
    Missile,
    BuckShot,
    MineThrower,
    OrbSlinger,
    CrissCross,
    Swarm,
}

impl EnemyType {
    pub fn spawn_child_with(
        &self,
        entity: Entity,
        commands: &mut Commands,
        server: &AssetServer,
        bundle: impl Bundle,
    ) -> Entity {
        let mut entity_commands = commands.spawn_empty();
        self.insert(&mut entity_commands, server, bundle);
        self.configure(&mut entity_commands);
        let id = entity_commands.id();
        commands.entity(entity).add_child(id);
        id
    }

    fn configure(&self, commands: &mut EntityCommands) {
        let mut rng = rand::rng();
        match self {
            Self::Gunner => commands.with_child((
                BulletModifiers {
                    rate: Rate::Factor(0.75),
                    ..Default::default()
                },
                SoloEmitter::player(),
            )),
            Self::Missile => commands
                .insert((
                    BackAndForth {
                        radius: rng.random_range(9.0..11.0),
                        speed: rng.random_range(2.2..3.4),
                    },
                    Angle(rng.random_range(0.0..2.0)),
                ))
                .with_child((
                    HomingEmitter::<Player>::player(),
                    TurnSpeed(55.),
                    BulletModifiers {
                        rate: Rate::Factor(0.25),
                        speed: 0.75,
                        ..Default::default()
                    },
                    MaxLifetime(Duration::from_secs_f32(5.0)),
                )),
            Self::MineThrower => commands
                .insert((
                    BackAndForth {
                        radius: rng.random_range(4.0..6.0),
                        speed: rng.random_range(1.2..2.4),
                    },
                    Angle(rng.random_range(0.0..2.0)),
                ))
                .with_child(MineEmitter::player()),
            Self::OrbSlinger => commands.with_child(OrbEmitter::player()),
            Self::BuckShot => commands.with_child((
                BuckShotEmitter::player().with_all(4, 1.5, 0.2),
                BulletModifiers {
                    speed: 0.4,
                    ..Default::default()
                },
            )),
            Self::CrissCross => commands.with_child(CrisscrossEmitter::player()),
            Self::Swarm => commands.insert(swarm::SwarmMovement).with_child((
                BulletModifiers {
                    rate: Rate::Factor(0.25),
                    speed: 0.5,
                    ..Default::default()
                },
                ProximityEmitter,
            )),
        };
    }

    fn insert(&self, commands: &mut EntityCommands, server: &AssetServer, bundle: impl Bundle) {
        commands.insert((*self, self.health(), self.sprite(server), bundle));
    }

    pub fn health(&self) -> Health {
        match self {
            Self::Gunner => Health::full(10.0),
            Self::Missile => Health::full(15.0),
            Self::MineThrower => Health::full(15.0),
            Self::OrbSlinger => Health::full(40.0),
            Self::BuckShot => Health::full(20.0),
            Self::CrissCross => Health::full(40.0),
            Self::Swarm => Health::full(1.),
        }
    }

    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Gunner => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(2, 3)),
            Self::Missile => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(3, 3)),
            Self::MineThrower => {
                assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(4, 3))
            }
            Self::OrbSlinger => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(3, 4)),
            Self::BuckShot => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(4, 4)),
            Self::CrissCross => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(2, 4)),
            Self::Swarm => assets::sprite_rect8(server, assets::SHIPS_PATH, UVec2::new(4, 0)),
        }
    }

    pub fn parts(&self) -> usize {
        match self {
            Self::Gunner => 1,
            Self::Missile => 2,
            Self::MineThrower => 3,
            Self::OrbSlinger => 8,
            Self::BuckShot => 8,
            Self::CrissCross => 6,
            Self::Swarm => {
                if rand::rng().random_bool(0.2) {
                    1
                } else {
                    0
                }
            }
        }
    }

    pub fn shield(&self) -> usize {
        match self {
            Self::Swarm => {
                if rand::rng().random() {
                    1
                } else {
                    0
                }
            }
            _ => self.parts(),
        }
    }
}

#[derive(Event)]
pub struct EnemyDeathEvent {
    entity: Entity,
    position: Vec2,
    enemy: EnemyType,
}

fn handle_death(
    q: Query<(Entity, &GlobalTransform, &EnemyType), With<Dead>>,
    mut commands: Commands,
    mut deaths: EventWriter<EnemyDeathEvent>,
    mut clusters: EventWriter<SpawnCluster>,
) {
    for (entity, transform, enemy_type) in q.iter() {
        let position = transform.compute_transform().translation.xy();
        deaths.write(EnemyDeathEvent {
            entity,
            position,
            enemy: *enemy_type,
        });
        clusters.write(SpawnCluster {
            parts: enemy_type.parts(),
            shield: enemy_type.shield(),
            position,
        });
        commands.entity(entity).despawn();
    }
}

fn death_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<EnemyDeathEvent>,
    atlas: Res<ExplosionLayout>,
) {
    for event in reader.read() {
        match event.enemy {
            EnemyType::Swarm => {
                commands.add_trauma(0.04);
            }
            _ => {
                commands.add_trauma(0.18);
            }
        }
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/melee.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));
        commands.spawn((
            Transform::from_translation(event.position.extend(100.)),
            Sprite::from_atlas_image(
                server.load("explosion.png"),
                TextureAtlas {
                    layout: atlas.0.clone(),
                    index: 0,
                },
            ),
            AnimationController::from_seconds(AnimationIndices::once_despawn(0..=9), 0.08),
        ));
    }
}

#[derive(Component)]
struct LowHealthEffects;

fn add_low_health_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    fire: Res<FireLayout>,
    sparks: Res<SparksLayout>,
    query: Query<(Entity, &Health), (With<EnemyType>, Without<LowHealthEffects>)>,
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
                .insert(LowHealthEffects)
                .with_children(|root| {
                    for dir in chosen.iter() {
                        root.spawn(fire_effect(
                            &mut rng,
                            &server,
                            &fire,
                            (dir.to_vec2() * DIST) + Vec2::Y * Y_OFFSET,
                        ));
                    }
                    root.spawn(sparks_effect(&mut rng, &server, &sparks, Vec2::ZERO));
                });
        }
    }
}

fn fire_effect(
    rng: &mut ThreadRng,
    server: &AssetServer,
    layout: &FireLayout,
    position: Vec2,
) -> impl Bundle {
    (
        Transform::from_scale(Vec3::splat(0.2)).with_translation(position.extend(1.)),
        Sprite {
            image: server.load("fire_sparks.png"),
            texture_atlas: Some(TextureAtlas {
                layout: layout.0.clone(),
                index: 0,
            }),
            flip_x: rng.random_bool(0.5),
            ..Default::default()
        },
        AnimationController::from_seconds(
            AnimationIndices::repeating(0..=18),
            rng.random_range(0.025..0.05),
        ),
    )
}

fn sparks_effect(
    rng: &mut ThreadRng,
    server: &AssetServer,
    layout: &SparksLayout,
    position: Vec2,
) -> impl Bundle {
    (
        Transform::from_scale(Vec3::splat(0.2)).with_translation(position.extend(-1.)),
        Sprite {
            image: server.load("sparks.png"),
            texture_atlas: Some(TextureAtlas {
                layout: layout.0.clone(),
                index: 0,
            }),
            flip_x: rng.random_bool(0.5),
            ..Default::default()
        },
        AnimationController::from_seconds(
            AnimationIndices::repeating(0..=19),
            rng.random_range(0.025..0.0251),
        ),
    )
}
