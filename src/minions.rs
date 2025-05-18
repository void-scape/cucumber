use crate::asteroids::MaterialCluster;
use crate::auto_collider::ImageCollider;
use crate::bounds::WallDespawn;
use crate::bullet::emitter::{
    BulletModifiers, EmitterState, GattlingEmitter, MissileEmitter, Rate,
};
use crate::bullet::homing::{Heading, HomingRotate, HomingTarget, TurnSpeed};
use crate::bullet::{BulletTimer, Polarity};
use crate::color::HexColor;
use crate::effects::Blasters;
use crate::pickups::{Collectable, Material, PickupEvent, PowerUp, Weapon};
use crate::player::{AliveContext, NormalShot, PLAYER_SPEED, Player, PowerUpEvent, WeaponRack};
use crate::sprites::{CellSize, TiltSprite};
use crate::text::flash_text;
use crate::{GameState, Layer, RESOLUTION_SCALE, points};
use avian2d::prelude::*;
use bevy::color::palettes::css::LIGHT_BLUE;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_enhanced_input::prelude::*;
use bevy_optix::debug::DebugRect;
use bevy_seedling::prelude::*;
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind};
use bevy_tween::tween::IntoTarget;
use std::time::Duration;

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_gunners,
                spawn_gunners,
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
    //mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    //player: Single<Entity, With<Player>>,
    mut rack: ResMut<WeaponRack>,
) {
    //if input.just_pressed(KeyCode::KeyO) {
    //    commands.spawn((Miner, MinerLeader(*player)));
    //}
    //
    //if input.just_pressed(KeyCode::KeyP) {
    //    commands.spawn((Gunner, GunnerLeader(*player)));
    //}

    if input.just_pressed(KeyCode::Digit1) {
        rack.aquire(Weapon::Bullet);
    } else if input.just_pressed(KeyCode::Digit2) {
        rack.aquire(Weapon::Missile);
    } else if input.just_pressed(KeyCode::Digit3) {
        if let Some(selection) = rack.selection() {
            rack.expire(selection);
        }
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
    player: Single<(&CollidingEntities, &Transform), With<Player>>,
    materials: Query<&Material>,
    pickups: Query<&PowerUp>,
    mut power_ups: EventWriter<PowerUpEvent>,
    mut writer: EventWriter<PickupEvent>,
    time: Res<Time>,
    mut timer: Local<(Stopwatch, usize)>,
) {
    let (entities, transform) = player.into_inner();
    timer.0.tick(time.delta());

    for entity in entities
        .iter()
        .copied()
        .filter(|entity| pickups.get(*entity).is_ok())
    {
        power_ups.write(PowerUpEvent);
        commands.entity(entity).despawn();

        flash_text(
            &mut commands,
            &server,
            "POWER-UP",
            24.,
            ((transform.translation.xy() + Vec2::Y * 20.) * RESOLUTION_SCALE)
                .extend(points::POINT_TEXT_Z + 2.),
            HexColor(0x3dff6e),
        );
    }

    let mut despawned = HashSet::new();
    //for miner in miners.iter() {
    for (entity, mat) in entities
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

const SUCK_SPEED: f32 = 6.;
const SUCK_DIST: f32 = 30.;
const NO_SHOT_SUCK_DIST: f32 = crate::HEIGHT / 2.;

fn suck_materials(
    player: Single<(&Transform, &Actions<AliveContext>), With<Player>>,
    mut materials: Query<(&GlobalTransform, &mut Transform), (With<Collectable>, Without<Player>)>,
    time: Res<Time>,
) {
    let (transform, actions) = player.into_inner();
    let pp = transform.translation.xy();
    let threshold = if actions.action::<NormalShot>().state() == ActionState::Fired {
        SUCK_DIST
    } else {
        NO_SHOT_SUCK_DIST
    };

    for (gt, mut t) in materials.iter_mut() {
        let p = gt.compute_transform().translation.xy();
        let dist = p.distance(pp);

        if dist < threshold {
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
    Visibility,
    RigidBody::Kinematic,
    LinearVelocity,
    Blasters(const { &[Vec3::new(0., -4., -1.)] }),
    Transform::from_xyz(0., -crate::HEIGHT / 2. - 10., 0.),
)]
pub struct Gunner;

#[derive(Debug, Component)]
pub struct GunnerWeapon {
    pub weapon: Weapon,
    pub enabled: bool,
}

#[derive(Component)]
pub enum GunnerAnchor {
    Left,
    Right,
    Bottom,
}

#[derive(Component)]
struct GunnerEmitter;

const EASE_OUT_DUR: f32 = 3.;

fn spawn_gunners(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    actions: Single<&Actions<AliveContext>, With<Player>>,
    mut gunners: Query<(Entity, &mut GunnerWeapon, &Transform), With<Gunner>>,
    mut rack: ResMut<WeaponRack>,
) {
    if rack.is_changed() && gunners.is_empty() {
        if let Some(weapon) = rack.selection() {
            let enabled = actions.action::<NormalShot>().state() == ActionState::Fired;
            spawn_gunner(&mut commands, *player, weapon, GunnerAnchor::Left, enabled);
            spawn_gunner(
                &mut commands,
                *player,
                weapon,
                GunnerAnchor::Bottom,
                enabled,
            );
            spawn_gunner(&mut commands, *player, weapon, GunnerAnchor::Right, enabled);
        }
    } else if rack.is_changed() {
        match rack.selection() {
            Some(w) => {
                for (_, mut weapon, _) in gunners.iter_mut() {
                    weapon.enabled = actions.action::<NormalShot>().state() == ActionState::Fired;
                    weapon.weapon = w;
                }
            }
            None => {
                for (entity, _, transform) in gunners.iter() {
                    commands
                        .entity(entity)
                        .despawn_related::<Children>()
                        .remove::<(Gunner, GunnerWeapon, GunnerLeader, GunnerAnchor)>()
                        .insert((
                            CollisionLayers::new(Layer::Player, Layer::Bounds),
                            WallDespawn,
                        ))
                        .animation()
                        .insert_tween_here(
                            Duration::from_secs_f32(
                                transform
                                    .translation
                                    .xy()
                                    .distance(Vec2::NEG_Y * crate::HEIGHT / 1.8)
                                    / crate::HEIGHT
                                    * EASE_OUT_DUR,
                            ),
                            EaseKind::QuadraticIn,
                            entity.into_target().with(translation(
                                transform.translation,
                                Vec3::NEG_Y * crate::HEIGHT / 1.8,
                            )),
                        );
                }
            }
        }
    }
}

fn spawn_gunner(
    commands: &mut Commands,
    player: Entity,
    weapon: Weapon,
    anchor: GunnerAnchor,
    enabled: bool,
) {
    commands.spawn((
        Gunner,
        GunnerLeader(player),
        anchor,
        GunnerWeapon { weapon, enabled },
        TiltSprite {
            path: "ships.png",
            size: CellSize::Eight,
            //
            left: UVec2::new(0, 1),
            center: UVec2::new(1, 1),
            right: UVec2::new(2, 1),
        },
        children![
            (
                GunnerEmitter,
                Polarity::North,
                MissileEmitter,
                BulletModifiers {
                    damage: 0.6,
                    rate: Rate::Factor(1.8),
                    ..Default::default()
                },
                EmitterState { enabled: false },
            ),
            (
                GunnerEmitter,
                GattlingEmitter(0.25),
                BulletModifiers {
                    damage: 0.2,
                    ..Default::default()
                },
                EmitterState { enabled: false },
            )
        ],
    ));
}

fn update_gunners(
    player: Single<&Transform, With<Player>>,
    mut gunners: Query<(&Transform, &mut LinearVelocity, &GunnerAnchor), With<Gunner>>,
    weapons: Query<(&GunnerWeapon, &Children), Changed<GunnerWeapon>>,
    mut gattling: Query<
        (&mut EmitterState, &mut BulletTimer),
        (With<GunnerEmitter>, With<GattlingEmitter>),
    >,
    mut missile: Query<
        (&mut EmitterState, &mut BulletTimer),
        (
            With<GunnerEmitter>,
            With<MissileEmitter>,
            Without<GattlingEmitter>,
        ),
    >,
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

        if velocity.0.x != 0.0 && velocity.0.x.abs() < 1. {
            velocity.0.x = 0.;
        }
    }

    for (weapon, children) in weapons.iter() {
        let mut iter = gattling.iter_many_mut(children);
        while let Some((mut state, mut timer)) = iter.fetch_next() {
            state.enabled = weapon.weapon == Weapon::Bullet && weapon.enabled;
            if state.enabled {
                let duration = timer.timer.duration();
                timer.timer.set_elapsed(duration);
            }
        }

        let mut iter = missile.iter_many_mut(children);
        while let Some((mut state, mut timer)) = iter.fetch_next() {
            state.enabled = weapon.weapon == Weapon::Missile && weapon.enabled;
            if state.enabled {
                let duration = timer.timer.duration();
                timer.timer.set_elapsed(duration);
            }
        }
    }
}
