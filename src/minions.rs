use crate::Layer;
use crate::asteroids::MaterialCluster;
use crate::auto_collider::ImageCollider;
use crate::bullet::homing::{Heading, HomingRotate, HomingTarget, TurnSpeed};
use crate::pickups::Material;
use crate::player::Player;
use avian2d::prelude::*;
use bevy::color::palettes::css::LIGHT_BLUE;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (test, (miner_collect, update_miners).chain()));
    }
}

fn test(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    player: Single<Entity, With<Player>>,
) {
    if input.just_pressed(KeyCode::KeyO) {
        commands.spawn((Miner, Leader(*player)));
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
    miners: Query<&CollidingEntities, With<Miner>>,
    materials: Query<&Material>,
) {
    let mut despawned = HashSet::new();
    for miner in miners.iter() {
        for entity in miner
            .iter()
            .copied()
            .filter(|entity| materials.get(*entity).is_ok())
        {
            if despawned.insert(entity) {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component)]
#[relationship(relationship_target = Minions)]
pub struct Leader(Entity);

#[derive(Component)]
#[relationship_target(relationship = Leader)]
pub struct Minions(Vec<Entity>);

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
    CollisionLayers::new(Layer::Player, [Layer::Collectable]),
)]
pub struct Miner;
