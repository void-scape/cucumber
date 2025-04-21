use crate::{
    HEIGHT, assets,
    auto_collider::ImageCollider,
    bullet::{BulletRate, BulletSpeed, BulletTimer, emitter::SoloEmitter},
    health::{Dead, Health, HealthSet},
};
use bevy::prelude::*;
use bevy_tween::{
    combinator::tween,
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use physics::{Physics, prelude::*};
use rand::Rng;
use std::time::Duration;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_waves,
                spawn_formations,
                despawn_formations,
                (update_circle, update_figure8),
            )
                .chain(),
        )
        .add_systems(Physics, handle_death.after(HealthSet))
        .insert_resource(WaveController::new_delayed(
            3.,
            &[(Formation::Triangle, 8.), (Formation::Triangle, 0.)],
        ));
    }
}

#[derive(Clone, Copy, Component)]
#[require(Transform, Visibility)]
pub enum Formation {
    Triangle,
}

impl Formation {
    pub fn enemies(&self) -> &'static [(Enemy, Vec2, MovementPattern)] {
        match self {
            Self::Triangle => {
                const {
                    &[
                        (
                            Enemy::Common,
                            Vec2::new(-40., -40.),
                            MovementPattern::Circle,
                        ),
                        (Enemy::Common, Vec2::ZERO, MovementPattern::Figure8),
                        (Enemy::Common, Vec2::new(40., -40.), MovementPattern::Circle),
                    ]
                }
            }
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
}

#[derive(Resource)]
struct WaveController {
    seq: &'static [(Formation, f32)],
    timer: Timer,
    index: usize,
}

impl WaveController {
    pub fn new(seq: &'static [(Formation, f32)]) -> Self {
        Self::new_delayed(0., seq)
    }

    pub fn new_delayed(delay: f32, seq: &'static [(Formation, f32)]) -> Self {
        assert!(
            !seq.is_empty(),
            "tried to create `WaveController` with empty sequence"
        );
        Self {
            seq,
            timer: Timer::from_seconds(delay, TimerMode::Repeating),
            index: 0,
        }
    }

    pub fn tick(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }

    pub fn next(&mut self) -> Option<Formation> {
        if self.timer.just_finished() {
            match self.seq.get(self.index) {
                Some((formation, duration)) => {
                    info!("set timer: {}", duration);
                    self.timer.set_duration(Duration::from_secs_f32(*duration));
                    self.index += 1;
                    Some(*formation)
                }
                None => None,
            }
        } else {
            None
        }
    }
}

fn update_waves(mut commands: Commands, mut controller: ResMut<WaveController>, time: Res<Time>) {
    controller.tick(&time);
    if let Some(formation) = controller.next() {
        commands.spawn(formation);
    }
}

const LARGEST_SPRITE_SIZE: f32 = 16.;
const FORMATION_EASE_DUR: f32 = 2.;

fn spawn_formations(
    mut commands: Commands,
    server: Res<AssetServer>,
    new_formations: Query<(Entity, &Formation), Without<Children>>,
    formations: Query<(Entity, &Transform), (With<Formation>, With<Children>)>,
) {
    let mut additional_height = 0.;
    for (root, formation) in new_formations.iter() {
        additional_height += formation.height();

        let start_y = HEIGHT / 2. - formation.bottomy() + LARGEST_SPRITE_SIZE / 2.;
        let end_y = HEIGHT / 2. + formation.topy() - LARGEST_SPRITE_SIZE / 2.;

        let start = Vec3::new(0., start_y, 0.);
        let end = Vec3::new(0., end_y - 20., 0.);

        commands.animation().insert(tween(
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
                (Transform::from_translation(position.extend(0.)),),
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
    formations: Query<(Entity, &Children), With<Formation>>,
    enemies: Query<&Enemy>,
) {
    for (entity, children) in formations.iter() {
        if children.iter().all(|entity| enemies.get(*entity).is_err()) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Transform, Velocity, Visibility, layers::Enemy, ImageCollider)]
pub enum Enemy {
    Common,
}

impl Enemy {
    pub fn spawn_child_with(
        &self,
        entity: Entity,
        commands: &mut Commands,
        server: &AssetServer,
        movement: MovementPattern,
        bundle: impl Bundle,
    ) {
        let mut entity_commands = commands.spawn_empty();
        self.insert(&mut entity_commands, server, movement, bundle);
        self.insert_emitter(&mut entity_commands);
        let id = entity_commands.id();
        commands.entity(entity).add_child(id);
    }

    fn insert_emitter(&self, commands: &mut EntityCommands) {
        commands.with_child((SoloEmitter::<layers::Player>::new(),));
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
            self.bullets(),
            BulletRate(0.20),
            BulletSpeed(0.5),
            bundle,
        ));
    }

    pub fn health(&self) -> Health {
        match self {
            Self::Common => Health::full(10),
        }
    }

    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Common => assets::sprite_rect16(server, assets::SHIPS_PATH, UVec2::new(2, 3)),
        }
    }

    pub fn bullets(&self) -> BulletTimer {
        match self {
            Self::Common => BulletTimer {
                timer: Timer::new(Duration::from_millis(1500), TimerMode::Repeating),
            },
        }
    }

    pub fn configure_movement(&self, commands: &mut EntityCommands, movement: MovementPattern) {
        let mut rng = rand::rng();
        match self {
            Self::Common => match movement {
                MovementPattern::Circle => {
                    commands.insert((
                        Circle {
                            radius: rng.random_range(9.0..11.0),
                            speed: rng.random_range(0.04..0.06),
                        },
                        Angle(rng.random_range(0.0..2.0)),
                    ));
                }
                MovementPattern::Figure8 => {
                    commands.insert((
                        Figure8 {
                            radius: rng.random_range(18.0..22.0),
                            speed: rng.random_range(0.04..0.06),
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
    Circle,
    Figure8,
}

#[derive(Component)]
#[require(Angle)]
struct Circle {
    radius: f32,
    speed: f32,
}

#[derive(Default, Component)]
struct Angle(f32);

#[derive(Component)]
struct Center(Vec2);

fn update_circle(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<Circle>, Without<Center>)>,
    mut query: Query<(&Circle, &Center, &mut Angle, &mut Transform)>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (circle, center, mut angle, mut transform) in query.iter_mut() {
        transform.translation.x = center.0.x + circle.radius * angle.0.cos();
        transform.translation.y = center.0.y + circle.radius * angle.0.sin();
        angle.0 += circle.speed;
        if angle.0 >= std::f32::consts::PI * 2. {
            angle.0 = 0.;
        }
    }
}

#[derive(Component)]
#[require(Angle)]
struct Figure8 {
    radius: f32,
    speed: f32,
}

fn update_figure8(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<Figure8>, Without<Center>)>,
    mut query: Query<(&mut Figure8, &Center, &mut Angle, &mut Transform)>,
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

        angle.0 += figure8.speed;
        if angle.0 >= std::f32::consts::TAU {
            angle.0 = 0.;
        }
    }
}

fn handle_death(q: Query<(Entity, &Enemy), With<Dead>>, mut commands: Commands) {
    for (entity, _enemy) in q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
