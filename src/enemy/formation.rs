use super::{EnemyType, movement::MovementPattern};
use crate::{GameState, HEIGHT};
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_tween::{
    combinator::tween,
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::time::Duration;

pub struct FormationPlugin;

impl Plugin for FormationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(
                Update,
                (spawn_formations, despawn_formations)
                    .in_set(FormationSet)
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct FormationSet;

fn restart(mut commands: Commands, formations: Query<Entity, With<Formation>>) {
    for entity in formations.iter() {
        commands
            .entity(entity)
            // there are two relationships here, and when you don't specify `Children`, `Units`
            // will panic as entities are deleted
            .despawn_related::<Children>()
            .despawn();
    }
}

#[derive(Debug, Clone, Copy, Component)]
#[require(Transform, Visibility)]
pub struct Formation(&'static [(EnemyType, Vec2, MovementPattern)]);

pub const TRIANGLE: Formation = Formation(&[
    (
        EnemyType::Gunner,
        Vec2::new(-40., -40.),
        MovementPattern::Circle,
    ),
    (EnemyType::Gunner, Vec2::ZERO, MovementPattern::Figure8),
    (
        EnemyType::Gunner,
        Vec2::new(40., -40.),
        MovementPattern::Circle,
    ),
]);

pub const ROW: Formation = Formation(&[
    (
        EnemyType::Missile,
        Vec2::new(30., 0.),
        MovementPattern::BackAndForth,
    ),
    (
        EnemyType::Missile,
        Vec2::new(-30., 0.),
        MovementPattern::BackAndForth,
    ),
]);

pub const MINE_THROWER: Formation = Formation(&[(
    EnemyType::MineThrower,
    Vec2::ZERO,
    MovementPattern::BackAndForth,
)]);

pub const ORB_SLINGER: Formation = Formation(&[
    (EnemyType::OrbSlinger, Vec2::ZERO, MovementPattern::None),
    (
        EnemyType::OrbSlinger,
        Vec2::new(40., -40.),
        MovementPattern::None,
    ),
]);

impl Formation {
    pub fn len(&self) -> usize {
        self.enemies().len()
    }

    pub fn enemies(&self) -> &'static [(EnemyType, Vec2, MovementPattern)] {
        self.0
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

    //pub fn drop_pickup_heuristic(&self) -> bool {
    //    const ENEMY_WEIGHT: f64 = 0.40;
    //
    //    let heuristic = self
    //        .enemies()
    //        .iter()
    //        .map(|(enemy, _, _)| {
    //            ENEMY_WEIGHT
    //                * match enemy {
    //                    EnemyType::Gunner => 0.75,
    //                    EnemyType::Missile => 0.85,
    //                }
    //        })
    //        .sum();
    //    info!("`{self:?}` drop heuristic: {heuristic:.2}");
    //    rand::rng().random_bool(heuristic)
    //}
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

        additional_height += formation.height() + PADDING;

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
    for (entity, _formation, _deaths) in formations.iter() {
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
