use self::{
    formation::{
        Formation, FormationPlugin, FormationSet, MINE_THROWER, ORB_SLINGER, ROW, TRIANGLE,
    },
    movement::*,
};
use crate::{
    GameState, Layer,
    animation::{AnimationController, AnimationIndices},
    assets,
    asteroids::{AsteroidSpawner, SpawnCluster},
    atlas_layout,
    bullet::{
        Destructable, Direction,
        emitter::{BulletModifiers, HomingEmitter, MineEmitter, OrbEmitter, SoloEmitter},
        homing::TurnSpeed,
    },
    health::{Dead, Health},
    miniboss,
    player::Player,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_optix::shake::TraumaCommands;
use bevy_seedling::{
    prelude::Volume,
    sample::{PlaybackSettings, SamplePlayer},
};
use bevy_sequence::combinators::delay::run_after;
use rand::{Rng, rngs::ThreadRng, seq::IteratorRandom};
use std::time::Duration;
use strum::IntoEnumIterator;

pub mod formation;
pub mod movement;

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
            .add_systems(OnEnter(GameState::StartGame), start_waves)
            .add_systems(
                Update,
                (
                    update_waves.before(FormationSet),
                    (add_low_health_effects, death_effects, handle_death),
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );
    }
}

atlas_layout!(FireLayout, init_fire_layout, 96, 4, 5);
atlas_layout!(SparksLayout, init_sparks_layout, 150, 5, 6);
atlas_layout!(ExplosionLayout, init_explosion_layout, 64, 10, 1);
atlas_layout!(CruiserExplosion, init_cruiser_explosion_layout, 128, 14, 1);

fn start_waves(mut commands: Commands) {
    info!("start waves");
    commands.insert_resource(WaveController::new_delayed(
        START_DELAY,
        &[
            (ORB_SLINGER, 0.),
            //(TRIANGLE, 16.),
            //(ROW, 8.),
            //(MINE_THROWER, 8.),
            //(TRIANGLE, 16.),
            //(ROW, 0.),
            //(MINE_THROWER, 8.),
        ],
    ));
}

#[derive(Resource)]
struct WaveController {
    seq: &'static [(Formation, f32)],
    timer: Timer,
    index: usize,
    finished: bool,
}

impl WaveController {
    pub fn new_delayed(delay: f32, seq: &'static [(Formation, f32)]) -> Self {
        Self {
            seq,
            timer: Timer::from_seconds(delay, TimerMode::Repeating),
            index: 0,
            finished: false,
        }
    }

    pub fn tick(&mut self, time: &Time<Physics>, formations_empty: bool) {
        self.timer.tick(time.delta());
        //if formations_empty {
        //    self.timer.set_elapsed(self.timer.duration());
        //}
    }

    pub fn next(&mut self) -> Option<Formation> {
        if self.timer.just_finished() {
            match self.seq.get(self.index) {
                Some((formation, duration)) => {
                    self.timer.set_duration(Duration::from_secs_f32(*duration));
                    self.index += 1;
                    Some(*formation)
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

fn update_waves(
    mut commands: Commands,
    controller: Option<ResMut<WaveController>>,
    formations: Query<&Formation>,
    time: Res<Time<Physics>>,
    mut asteroids: ResMut<AsteroidSpawner>,
) {
    let Some(mut controller) = controller else {
        return;
    };

    controller.tick(&time, formations.is_empty());
    if let Some(formation) = controller.next() {
        commands.spawn(formation);
    }

    if controller.finished() && formations.is_empty() {
        asteroids.0 = false;
        commands.remove_resource::<WaveController>();
        info!("ran out of formations, spawning boss");
        run_after(
            Duration::from_secs_f32(5.),
            miniboss::spawn_boss,
            &mut commands,
        );
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
    MineThrower,
    OrbSlinger,
}

impl EnemyType {
    pub fn spawn_child_with(
        &self,
        entity: Entity,
        commands: &mut Commands,
        server: &AssetServer,
        movement: MovementPattern,
        bundle: impl Bundle,
    ) -> Entity {
        let mut entity_commands = commands.spawn_empty();
        self.insert(&mut entity_commands, server, movement, bundle);
        self.insert_emitter(&mut entity_commands);
        let id = entity_commands.id();
        commands.entity(entity).add_child(id);
        id
    }

    fn insert_emitter(&self, commands: &mut EntityCommands) {
        match self {
            Self::Gunner => commands.with_child(SoloEmitter::player()),
            Self::Missile => {
                commands.with_child((HomingEmitter::<Player>::player(), TurnSpeed(60.)))
            }
            Self::MineThrower => commands.with_child(MineEmitter::player()),
            Self::OrbSlinger => commands.with_child(OrbEmitter::player()),
        };
    }

    fn insert(
        &self,
        commands: &mut EntityCommands,
        server: &AssetServer,
        movement: MovementPattern,
        bundle: impl Bundle,
    ) {
        self.configure_movement(commands, movement);
        commands.insert((*self, self.health(), self.sprite(server), bundle));
    }

    pub fn health(&self) -> Health {
        match self {
            Self::Gunner => Health::full(10.0),
            Self::Missile => Health::full(15.0),
            Self::MineThrower => Health::full(15.0),
            Self::OrbSlinger => Health::full(40.0),
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
        }
    }

    pub fn materials(&self) -> usize {
        match self {
            Self::Gunner => 1,
            Self::Missile => 2,
            Self::MineThrower => 3,
            Self::OrbSlinger => 8,
        }
    }
}

#[derive(Event)]
pub struct EnemyDeathEvent {
    entity: Entity,
    position: Vec2,
}

fn handle_death(
    q: Query<(Entity, &GlobalTransform, &EnemyType), With<Dead>>,
    mut commands: Commands,
    mut deaths: EventWriter<EnemyDeathEvent>,
    mut clusters: EventWriter<SpawnCluster>,
) {
    for (entity, transform, enemy_type) in q.iter() {
        let position = transform.compute_transform().translation.xy();
        deaths.write(EnemyDeathEvent { entity, position });
        clusters.write(SpawnCluster {
            materials: enemy_type.materials(),
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
        commands.add_trauma(0.18);
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
