use crate::{
    GameState, HEIGHT, Layer,
    animation::{AnimationController, AnimationIndices},
    assets, atlas_layout,
    bullet::{
        BulletRate, BulletSpeed, Destructable, Direction,
        emitter::{DualEmitter, HomingEmitter, SoloEmitter},
        homing::TurnSpeed,
    },
    health::{Dead, Health},
    miniboss, pickups,
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
    sample::{PlaybackSettings, SamplePlayer},
};
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::{
    combinator::tween,
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use rand::{Rng, rngs::ThreadRng, seq::IteratorRandom};
use std::time::Duration;
use strum::IntoEnumIterator;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDeathEvent>()
            .add_systems(
                Startup,
                (
                    init_fire_layout,
                    init_sparks_layout,
                    init_explosion_layout,
                    init_cruiser_explosion_layout,
                ),
            );
            //.add_systems(OnEnter(GameState::StartGame), start_waves)
            //.add_systems(
            //    Update,
            //    (
            //        update_waves,
            //        spawn_formations,
            //        despawn_formations,
            //        (add_low_health_effects, death_effects, handle_death),
            //    )
            //        .chain()
            //        .run_if(in_state(GameState::Game)),
            //)
            //.add_systems(
            //    FixedUpdate,
            //    (update_back_and_forth, update_circle, update_figure8)
            //        .run_if(in_state(GameState::Game)),
            //);
    }
}

atlas_layout!(FireLayout, init_fire_layout, 96, 4, 5);
atlas_layout!(SparksLayout, init_sparks_layout, 150, 5, 6);
atlas_layout!(ExplosionLayout, init_explosion_layout, 64, 10, 1);
atlas_layout!(CruiserExplosion, init_cruiser_explosion_layout, 128, 14, 1);

fn start_waves(mut commands: Commands) {
    info!("start waves");
    commands.insert_resource(WaveController::new_delayed(
        0.,
        &[
            (Formation::Triangle, 8.),
            (Formation::Row, 8.),
            (Formation::Triangle, 8.),
            (Formation::Row, 0.),
        ],
    ));
}

#[derive(Debug, Clone, Copy, Component)]
#[require(Transform, Visibility)]
pub enum Formation {
    Triangle,
    Row,
}

const TRIANGLE: &[(EnemyType, Vec2, MovementPattern)] = &[
    (
        EnemyType::Common,
        Vec2::new(-40., -40.),
        MovementPattern::Circle,
    ),
    (EnemyType::Common, Vec2::ZERO, MovementPattern::Figure8),
    (
        EnemyType::Common,
        Vec2::new(40., -40.),
        MovementPattern::Circle,
    ),
];

const ROW: &[(EnemyType, Vec2, MovementPattern)] = &[
    (
        EnemyType::Uncommon,
        Vec2::new(30., 0.),
        MovementPattern::BackAndForth,
    ),
    (
        EnemyType::Uncommon,
        Vec2::new(-30., 0.),
        MovementPattern::BackAndForth,
    ),
];

impl Formation {
    pub fn len(&self) -> usize {
        self.enemies().len()
    }

    pub fn enemies(&self) -> &'static [(EnemyType, Vec2, MovementPattern)] {
        match self {
            Self::Triangle => TRIANGLE,
            Self::Row => ROW,
        }
    }

    pub fn height(&self) -> f32 {
        self.topy() - self.bottomy()
    }

    pub fn bottomy(&self) -> f32 {
        debug_assert!(!self.enemies().is_empty());
        self.enemies()
            .iter()
            .map(|(_, pos, _)| pos.y)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap()
    }

    pub fn topy(&self) -> f32 {
        debug_assert!(!self.enemies().is_empty());
        self.enemies()
            .iter()
            .map(|(_, pos, _)| pos.y)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap()
    }

    pub fn drop_pickup_heuristic(&self) -> bool {
        const ENEMY_WEIGHT: f64 = 0.40;

        let heuristic = self
            .enemies()
            .iter()
            .map(|(enemy, _, _)| {
                ENEMY_WEIGHT
                    * match enemy {
                        EnemyType::Common => 0.75,
                        EnemyType::Uncommon => 0.85,
                    }
            })
            .sum();
        info!("`{self:?}` drop heuristic: {heuristic:.2}");
        rand::rng().random_bool(heuristic)
    }
}

#[derive(Component)]
#[relationship(relationship_target = Units)]
#[component(on_remove = on_remove_platoon)]
struct Platoon(Entity);

fn on_remove_platoon(mut world: DeferredWorld, ctx: HookContext) {
    let platoon = world.entity(ctx.entity).get::<Platoon>().unwrap().0;
    let position = world
        .entity(ctx.entity)
        .get::<GlobalTransform>()
        .unwrap()
        .compute_transform()
        .translation
        .xy();

    world
        .commands()
        .entity(platoon)
        .entry::<UnitDeaths>()
        .and_modify(move |mut deaths| deaths.death_position(position));
}

#[derive(Component)]
#[relationship_target(relationship = Platoon)]
#[require(UnitDeaths)]
struct Units(Vec<Entity>);

#[derive(Default, Component)]
struct UnitDeaths(Vec<Vec2>);

impl UnitDeaths {
    pub fn death_position(&mut self, position: Vec2) {
        self.0.push(position);
    }

    pub fn last_death_position(&self) -> Option<Vec2> {
        self.0.last().copied()
    }
}

#[derive(Resource)]
struct WaveController {
    seq: &'static [(Formation, f32)],
    timer: Timer,
    index: usize,
    finished: bool,
}

impl WaveController {
    pub fn new(seq: &'static [(Formation, f32)]) -> Self {
        Self::new_delayed(0., seq)
    }

    pub fn new_delayed(delay: f32, seq: &'static [(Formation, f32)]) -> Self {
        Self {
            seq,
            timer: Timer::from_seconds(delay, TimerMode::Repeating),
            index: 0,
            finished: false,
        }
    }

    pub fn tick(&mut self, time: &Time<Physics>) {
        self.timer.tick(time.delta());
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
    mut controller: ResMut<WaveController>,
    formations: Query<&Formation>,
    time: Res<Time<Physics>>,
    mut finished: Local<bool>,
) {
    if *finished {
        return;
    }

    controller.tick(&time);
    if let Some(formation) = controller.next() {
        commands.spawn(formation);
    }

    if controller.finished() && formations.is_empty() {
        info!("ran out of formations, spawning boss");
        run_after(
            Duration::from_secs_f32(5.),
            miniboss::spawn_boss,
            &mut commands,
        );
        *finished = true;
    }
}

const LARGEST_SPRITE_SIZE: f32 = 16.;
const PADDING: f32 = LARGEST_SPRITE_SIZE;
const FORMATION_EASE_DUR: f32 = 2.;

fn spawn_formations(
    mut commands: Commands,
    server: Res<AssetServer>,
    new_formations: Query<(Entity, &Formation), Without<UnitDeaths>>,
    formations: Query<(Entity, &Transform), (With<Formation>, With<Units>)>,
) {
    let mut additional_height = 0.;
    for (root, formation) in new_formations.iter() {
        info!("spawn new formation");

        additional_height += formation.height() - PADDING;

        let start_y = HEIGHT / 2. - formation.bottomy() + LARGEST_SPRITE_SIZE / 2.;
        let end_y = HEIGHT / 2. + formation.topy() - LARGEST_SPRITE_SIZE / 2.;

        let start = Vec3::new(0., start_y, 0.);
        let end = Vec3::new(0., end_y - 20., 0.);

        commands.entity(root).animation().insert(tween(
            Duration::from_secs_f32(FORMATION_EASE_DUR),
            EaseKind::SineOut,
            root.into_target().with(translation(start, end)),
        ));

        commands
            .entity(root)
            .insert(Transform::from_translation(start));

        for (enemy, position, movement) in formation.enemies().iter() {
            enemy.spawn_child_with(
                root,
                &mut commands,
                &server,
                *movement,
                (
                    Transform::from_translation(position.extend(0.)),
                    Platoon(root),
                ),
            );
        }
    }

    if additional_height > 0. {
        for (entity, transform) in formations.iter() {
            commands.animation().insert(tween(
                Duration::from_secs_f32(FORMATION_EASE_DUR),
                EaseKind::SineOut,
                entity.into_target().with(translation(
                    transform.translation,
                    Vec3::new(0., additional_height, 0.),
                )),
            ));
        }
    }
}

fn despawn_formations(
    mut commands: Commands,
    formations: Query<(Entity, &Formation, &UnitDeaths), Without<Units>>,
) {
    for (entity, formation, deaths) in formations.iter() {
        info!("despawning formation");
        commands.entity(entity).despawn();

        //if formation.drop_pickup_heuristic() {
        //    let mut commands = commands.spawn_empty();
        //    let position = deaths.last_death_position().unwrap();
        //
        //    pickups::spawn_random_pickup(
        //        &mut commands,
        //        (
        //            Transform::from_translation(position.extend(0.)),
        //            pickups::velocity(),
        //        ),
        //    );
        //}
    }
}

#[derive(Clone, Copy, Component, PartialEq, Eq, Hash)]
#[require(
    Transform,
    Visibility,
    LinearVelocity,
    Destructable,
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet]),
)]
pub enum EnemyType {
    Common,
    Uncommon,
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
            Self::Common => commands.with_child(SoloEmitter::player()),
            Self::Uncommon => {
                commands.with_child((HomingEmitter::<Player>::player(), TurnSpeed(60.)))
            }
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
        commands.insert((
            *self,
            self.health(),
            self.sprite(server),
            BulletRate(0.20),
            BulletSpeed(0.5),
            bundle,
        ));
    }

    pub fn health(&self) -> Health {
        match self {
            Self::Common => Health::full(10.0),
            Self::Uncommon => Health::full(15.0),
        }
    }

    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Common => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(2, 3)),
            Self::Uncommon => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(3, 3)),
        }
    }

    pub fn configure_movement(&self, commands: &mut EntityCommands, movement: MovementPattern) {
        let mut rng = rand::rng();
        match self {
            Self::Common | Self::Uncommon => match movement {
                MovementPattern::BackAndForth => {
                    commands.insert((
                        BackAndForth {
                            radius: rng.random_range(9.0..11.0),
                            speed: rng.random_range(2.2..3.4),
                        },
                        Angle(rng.random_range(0.0..2.0)),
                    ));
                }
                MovementPattern::Circle => {
                    commands.insert((
                        Circle {
                            radius: rng.random_range(9.0..11.0),
                            speed: rng.random_range(2.4..3.6),
                        },
                        Angle(rng.random_range(0.0..2.0)),
                    ));
                }
                MovementPattern::Figure8 => {
                    commands.insert((
                        Figure8 {
                            radius: rng.random_range(18.0..22.0),
                            speed: rng.random_range(2.4..3.6),
                        },
                        Angle(rng.random_range(0.0..2.0)),
                    ));
                }
            },
        }
    }
}

#[derive(Clone, Copy)]
pub enum MovementPattern {
    BackAndForth,
    Circle,
    Figure8,
}

#[derive(Component)]
#[require(Angle)]
struct BackAndForth {
    radius: f32,
    speed: f32,
}

fn update_back_and_forth(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<BackAndForth>, Without<Center>)>,
    mut query: Query<(&BackAndForth, &Center, &mut Angle, &mut Transform)>,
    time: Res<Time<Physics>>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (baf, center, mut angle, mut transform) in query.iter_mut() {
        transform.translation.x = center.0.x + baf.radius * angle.0.cos();
        angle.0 += baf.speed * time.delta_secs();
        if angle.0 >= std::f32::consts::PI * 2. {
            angle.0 = 0.;
        }
    }
}

#[derive(Component)]
#[require(Angle)]
struct Circle {
    radius: f32,
    speed: f32,
}

#[derive(Default, Component)]
pub struct Angle(pub f32);

#[derive(Component)]
pub struct Center(pub Vec2);

fn update_circle(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<Circle>, Without<Center>)>,
    mut query: Query<(&Circle, &Center, &mut Angle, &mut Transform)>,
    time: Res<Time<Physics>>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (circle, center, mut angle, mut transform) in query.iter_mut() {
        transform.translation.x = center.0.x + circle.radius * angle.0.cos();
        transform.translation.y = center.0.y + circle.radius * angle.0.sin();
        angle.0 += circle.speed * time.delta_secs();
        if angle.0 >= std::f32::consts::PI * 2. {
            angle.0 = 0.;
        }
    }
}

#[derive(Component)]
#[require(Angle)]
pub struct Figure8 {
    pub radius: f32,
    pub speed: f32,
}

fn update_figure8(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<Figure8>, Without<Center>)>,
    mut query: Query<(&mut Figure8, &Center, &mut Angle, &mut Transform)>,
    time: Res<Time<Physics>>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (figure8, center, mut angle, mut transform) in query.iter_mut() {
        let t = angle.0;
        transform.translation.x = center.0.x + figure8.radius * t.sin();
        transform.translation.y = center.0.y + figure8.radius * t.sin() * t.cos();

        angle.0 += figure8.speed * time.delta_secs();
        if angle.0 >= std::f32::consts::TAU {
            angle.0 = 0.;
        }
    }
}

#[derive(Event)]
struct EnemyDeathEvent {
    entity: Entity,
    position: Vec2,
}

fn handle_death(
    q: Query<(Entity, &GlobalTransform), (With<Dead>, With<EnemyType>)>,
    mut commands: Commands,
    mut writer: EventWriter<EnemyDeathEvent>,
) {
    for (entity, transform) in q.iter() {
        writer.write(EnemyDeathEvent {
            entity,
            position: transform.compute_transform().translation.xy(),
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
