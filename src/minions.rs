use crate::asteroids::MaterialCluster;
use crate::auto_collider::ImageCollider;
use crate::bullet::Polarity;
use crate::bullet::emitter::{BulletModifiers, GattlingEmitter, HomingEmitter, PlayerEmitter};
use crate::bullet::homing::{Heading, HomingRotate, HomingTarget, TurnSpeed};
use crate::enemy::Enemy;
use crate::pickups::{Material, PickupEvent, Weapon};
use crate::player::{PLAYER_SPEED, Player};
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

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_gunners,
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
    for (entity, mat) in player
        .iter()
        .copied()
        .flat_map(|entity| materials.get(entity).map(|mat| (entity, mat)))
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
            writer.write(PickupEvent::Material(*mat));
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
pub struct GunnerLeader(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = GunnerLeader, linked_spawn)]
pub struct Gunners(Vec<Entity>);

#[derive(Component)]
#[require(
    Transform,
    RigidBody::Kinematic,
    LinearVelocity,
    DebugRect::from_size_color(Vec2::splat(4.), LIGHT_GREEN)
)]
pub struct Gunner;

#[derive(Component)]
pub struct GunnerWeapon(pub Weapon);

#[derive(Component)]
pub enum GunnerAnchor {
    Left,
    Right,
    Bottom,
}

fn update_gunners(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    mut gunners: Query<(&Transform, &mut LinearVelocity, &GunnerAnchor), With<Gunner>>,
    weapons: Query<(Entity, &GunnerWeapon), Changed<GunnerWeapon>>,
) {
    let pp = player.translation.xy();
    for (transform, mut velocity, anchor) in gunners.iter_mut() {
        let p = transform.translation.xy();
        let anchor = match anchor {
            GunnerAnchor::Left => Vec2::new(-15., 0.),
            GunnerAnchor::Right => Vec2::new(15., 0.),
            GunnerAnchor::Bottom => Vec2::new(0., -15.),
        };

        const LAG: f32 = 5.;
        let to_player = (pp + anchor - p).clamp_length(0., LAG) / LAG;
        velocity.0 = to_player * PLAYER_SPEED;
    }

    for (entity, weapon) in weapons.iter() {
        commands.entity(entity).despawn_related::<Children>();
        match weapon.0 {
            Weapon::Missile => {
                commands.entity(entity).with_child((
                    PlayerEmitter,
                    Polarity::North,
                    HomingEmitter::<Enemy>::enemy(),
                    BulletModifiers {
                        damage: 0.33,
                        ..Default::default()
                    },
                ));
            }
            Weapon::Bullet => {
                commands.entity(entity).with_child((
                    GattlingEmitter,
                    BulletModifiers {
                        damage: 0.33,
                        ..Default::default()
                    },
                ));
            }
            _ => {}
        }
    }
}
