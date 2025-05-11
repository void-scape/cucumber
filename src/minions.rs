use crate::asteroids::MaterialCluster;
use crate::auto_collider::ImageCollider;
use crate::bullet::Polarity;
use crate::bullet::emitter::SoloEmitter;
use crate::bullet::homing::{Heading, HomingRotate, HomingTarget, TurnSpeed};
use crate::enemy::movement::{Angle, Center, Figure8};
use crate::pickups::{Material, PickupEvent};
use crate::player::Player;
use crate::tween::Tween;
use crate::{GameState, Layer};
use avian2d::prelude::*;
use bevy::color::palettes::css::{LIGHT_BLUE, LIGHT_GREEN};
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_optix::debug::DebugRect;
use bevy_seedling::prelude::*;
use bevy_sequence::prelude::*;
use bevy_tween::combinator::tween;
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::EaseKind;
use bevy_tween::tween::IntoTarget;
use rand::Rng;
use std::time::Duration;

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_gunner_formation,
                suck_materials,
                (miner_collect, update_miners).chain(),
            )
                .run_if(in_state(GameState::Game)),
        );

        #[cfg(debug_assertions)]
        app.add_systems(Update, test_spawn);
    }
}

fn test_spawn(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    player: Single<Entity, With<Player>>,
) {
    if input.just_pressed(KeyCode::KeyO) {
        commands.spawn((Miner, MinerLeader(*player)));
    }

    if input.just_pressed(KeyCode::KeyP) {
        commands.spawn((Gunner, GunnerLeader(*player)));
    }
}

#[derive(Component)]
#[relationship(relationship_target = ClusterAssignments)]
struct Assigned(Entity);

#[derive(Component)]
#[relationship_target(relationship = Assigned)]
struct ClusterAssignments(Vec<Entity>);

#[derive(Component)]
struct BoundToPlayer;

#[derive(Component)]
struct MinerTarget;

fn update_miners(
    mut commands: Commands,
    miners: Query<(Entity, &Transform, Option<&BoundToPlayer>), (With<Miner>, Without<Assigned>)>,
    assigned_miners: Query<
        (Entity, &Assigned, &Transform),
        (With<Miner>, Without<HomingTarget>, Without<BoundToPlayer>),
    >,
    player: Single<Entity, With<Player>>,
    clusters: Query<
        (Entity, &Transform, &Children, Option<&ClusterAssignments>),
        With<MaterialCluster>,
    >,
    materials: Query<(Entity, &GlobalTransform, Option<&MinerTarget>), With<Material>>,
) {
    for (miner, transform, bound) in miners.iter() {
        let p = transform.translation.xy();
        if bound.is_none() && clusters.is_empty() {
            commands
                .entity(miner)
                .insert((BoundToPlayer, HomingTarget(*player)));
            continue;
        } else if bound.is_some() && !clusters.is_empty() {
            commands
                .entity(miner)
                .remove::<(BoundToPlayer, HomingTarget)>();
        }

        if let Some((cluster, _, _, _)) = clusters
        .iter()
        .sort_unstable_by_key::<(Entity, &Transform, &Children, Option<&ClusterAssignments>), (i64, usize)>(
            |(_, transform, _, assignments)| {
                (
                    p.distance(transform.translation.xy()).round() as i64,
                    assignments.map(|a| a.0.len()).unwrap_or_default(),
                )
            },
        ).next() {
            commands.entity(miner).insert(Assigned(cluster));
        }
    }

    let mut targeted = HashSet::new();
    for (miner, assignment, transform) in assigned_miners.iter() {
        if let Ok((_, _, children, _)) = clusters.get(assignment.0) {
            let p = transform.translation.xy();
            if let Some((closest, _, _)) = materials
                .iter_many(children)
                .filter(|(entity, _, target)| target.is_none() && !targeted.contains(entity))
                .min_by(|a, b| {
                    let a = p.distance(a.1.compute_transform().translation.xy());
                    let b = p.distance(b.1.compute_transform().translation.xy());
                    a.total_cmp(&b)
                })
            {
                commands.entity(miner).insert(HomingTarget(closest));
                commands.entity(closest).insert(MinerTarget);
                targeted.insert(closest);
            }
        }
    }
}

fn miner_collect(
    mut commands: Commands,
    server: Res<AssetServer>,
    //miners: Query<&CollidingEntities, With<Miner>>,
    player: Single<&CollidingEntities, With<Player>>,
    materials: Query<&Material>,
    mut writer: EventWriter<PickupEvent>,
    time: Res<Time>,
    mut timer: Local<(Stopwatch, usize)>,
) {
    timer.0.tick(time.delta());

    let mut despawned = HashSet::new();
    //for miner in miners.iter() {
    for entity in player
        .iter()
        .copied()
        .filter(|entity| materials.get(*entity).is_ok())
    {
        if despawned.insert(entity) {
            let speed = if timer.0.elapsed_secs() < 1. {
                timer.1 += 1;
                let speed = 1.0 + timer.1 as f64 * 0.05;
                speed
            } else {
                timer.1 = 0;
                1.0
            };
            timer.0.reset();

            commands.entity(entity).despawn();
            writer.write(PickupEvent::Material);
            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/click.wav")),
                PlaybackSettings {
                    volume: Volume::Linear(0.2),
                    ..Default::default()
                },
                PlaybackParams {
                    speed,
                    ..Default::default()
                },
            ));
        }
    }
    //}
}

const SUCK_SPEED: f32 = 4.;
const SUCK_DIST: f32 = 40.;

fn suck_materials(
    player: Single<&Transform, With<Player>>,
    mut materials: Query<(&GlobalTransform, &mut Transform), (With<Material>, Without<Player>)>,
    time: Res<Time>,
) {
    let pp = player.translation.xy();
    for (gt, mut t) in materials.iter_mut() {
        let p = gt.compute_transform().translation.xy();
        let dist = p.distance(pp);
        if dist < SUCK_DIST {
            t.translation += (pp - p).extend(0.) * SUCK_SPEED * time.delta_secs() * 20. / dist;
        }
    }
}

fn update_gunner_formation(
    mut commands: Commands,
    player_gunners: Single<&Gunners, (With<Player>, Changed<Gunners>)>,
    gunners: Query<(Entity, &Transform), With<Gunner>>,
) {
    let count = player_gunners.len();
    let width = crate::WIDTH - 32.;
    let half_width = width / 2.;
    let step = width / count as f32;
    let start_x = -half_width + step / 2.;

    let arc_height = 100.0;
    let formation = (0..count).map(|i| {
        let x = start_x + i as f32 * step;
        let normalized_x = x / half_width;
        let y = -arc_height * (normalized_x * normalized_x);
        Vec2::new(x, y)
    });

    for ((gunner, transform), position) in gunners.iter_many(&player_gunners.0).zip(formation) {
        let start = transform.translation;
        let end = position.extend(transform.translation.z);
        let dist = start.distance(end);

        commands.entity(gunner).remove::<(Figure8, Angle, Center)>();

        let frag = Tween(tween(
            Duration::from_secs_f32(dist / GUNNER_SPEED / 1.25),
            EaseKind::QuadraticInOut,
            gunner.into_target().with(translation(start, end)),
        ))
        .on_end(move |mut commands: Commands| {
            let mut rng = rand::rng();
            commands.entity(gunner).insert((
                Center(end.xy()),
                Figure8 {
                    radius: rng.random_range(18.0..22.0) / (count as f32 * 0.75),
                    speed: rng.random_range(2.4..3.6),
                },
                Angle(0.),
            ));
        })
        .always()
        .once();
        spawn_root(frag, &mut commands);
    }
}

#[derive(Component)]
#[relationship(relationship_target = Miners)]
pub struct MinerLeader(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = MinerLeader, linked_spawn)]
pub struct Miners(Vec<Entity>);

const MINER_SPEED: f32 = 40.;

#[derive(Component)]
#[require(
    Sensor,
    CollidingEntities,
    RigidBody::Kinematic,
    ImageCollider,
    LinearVelocity,
    DebugRect::from_size_color(Vec2::splat(4.), LIGHT_BLUE),
    Heading {
        direction: 0.,
        speed: MINER_SPEED,
    },
    TurnSpeed,
    HomingRotate,
    CollisionLayers::new(Layer::Miners, [Layer::Collectable]),
)]
pub struct Miner;

#[derive(Component)]
#[relationship(relationship_target = Gunners)]
pub struct GunnerLeader(Entity);

#[derive(Component)]
#[relationship_target(relationship = GunnerLeader, linked_spawn)]
pub struct Gunners(Vec<Entity>);

const GUNNER_SPEED: f32 = 40.;

#[derive(Component)]
#[require(
    Transform,
    RigidBody::Kinematic,
    LinearVelocity,
    DebugRect::from_size_color(Vec2::splat(4.), LIGHT_GREEN)
)]
#[component(on_add = Self::add_emitter)]
pub struct Gunner;

impl Gunner {
    fn add_emitter(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .with_child((SoloEmitter::enemy(), Polarity::North));
    }
}
