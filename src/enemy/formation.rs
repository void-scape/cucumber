use super::swarm::SwarmMovement;
use super::timeline::LARGEST_SPRITE_SIZE;
use super::{BuckShot, CrissCross, MineThrower, OrbSlinger, Swarm, timeline::WaveTimeline};
use super::{InvincibleLaserNode, WallShooter};
use crate::bullet::emitter::{BulletModifiers, EmitterDelay, LaserEmitter, WallEmitter};
use crate::pickups::{Pickup, Weapon};
use crate::tween::OnEnd;
use crate::{Avian, DespawnRestart, GameState, boss::gradius};
use avian2d::prelude::Physics;
use bevy::color::palettes::css::WHITE;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::interpolate::{rotation, sprite_color, translation};
use bevy_tween::prelude::*;
use bevy_tween::tween::apply_component_tween_system;
use std::f32;
use std::f32::consts::PI;

pub const DEFAULT_FORMATION_VEL: Vec2 = Vec2::new(0., -12.);

pub struct FormationPlugin;

impl Plugin for FormationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            despawn_formations
                .in_set(FormationSet)
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(Avian, update_formations)
        .add_tween_systems(apply_component_tween_system::<LaserEmitterTween>);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct FormationSet;

#[derive(Component)]
#[require(Transform, Visibility, DespawnRestart)]
pub struct FormationEntity(pub Vec2);

// We leak formations when they die until the game restarts, but this is fine
pub struct Formation {
    pub spawn: fn(&mut EntityCommands, &AssetServer),
    pub modifiers: Vec<Box<dyn FnMut(&mut EntityCommands) + Send + Sync>>,
    pub velocity: Vec2,
}

impl Formation {
    pub fn new(spawn: fn(&mut EntityCommands, &AssetServer)) -> Self {
        Self::with_velocity(DEFAULT_FORMATION_VEL, spawn)
    }

    pub fn with_velocity(velocity: Vec2, spawn: fn(&mut EntityCommands, &AssetServer)) -> Self {
        Self {
            spawn,
            velocity,
            modifiers: Vec::new(),
        }
    }

    pub fn with(
        mut self,
        modifier: impl FnMut(&mut EntityCommands) + Send + Sync + 'static,
    ) -> Self {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

#[derive(Component)]
struct DropOption(Weapon);

pub fn option(weapon: Weapon) -> impl FnMut(&mut EntityCommands) + 'static {
    move |commands| {
        commands.insert(DropOption(weapon));
    }
}

pub fn mine_thrower() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![(MineThrower, Platoon(formation.id()))]);
    })
}

pub fn quad_mine_thrower() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (
                MineThrower,
                Platoon(formation.id()),
                Transform::from_xyz(-20., 0., 0.)
            ),
            (
                MineThrower,
                Platoon(formation.id()),
                Transform::from_xyz(20., 0., 1.)
            ),
            (
                MineThrower,
                Platoon(formation.id()),
                Transform::from_xyz(-40., 10., 2.)
            ),
            (
                MineThrower,
                Platoon(formation.id()),
                Transform::from_xyz(40., 10., 3.)
            ),
        ]);
    })
}

pub fn double_buck_shot() -> Formation {
    Formation::new(|formation: &mut EntityCommands, server: &AssetServer| {
        formation.with_children(|root| {
            let platoon = root.target_entity();

            animate_entrance(
                &server,
                &mut root.commands(),
                (
                    ChildOf(platoon),
                    BuckShot,
                    Platoon(platoon),
                    Transform::from_xyz(-30., 0., 0.),
                ),
                None,
                1.5,
                Vec3::new(-crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                Vec3::new(-30., -10., 0.),
                Quat::from_rotation_z(PI / 4.) + Quat::from_rotation_x(PI / 4.),
                Quat::default(),
            );

            animate_entrance(
                &server,
                &mut root.commands(),
                (
                    ChildOf(platoon),
                    BuckShot,
                    Platoon(platoon),
                    Transform::from_xyz(-30., 0., 0.),
                ),
                Some(1.),
                1.5,
                Vec3::new(-crate::WIDTH / 2. - LARGEST_SPRITE_SIZE / 2., 0., 0.),
                Vec3::new(30., -10., 0.),
                Quat::from_rotation_z(PI / 3.) + Quat::from_rotation_x(PI / 4.),
                Quat::default(),
            );
        });
    })
}

pub fn double_wall() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (
                WallShooter,
                Platoon(formation.id()),
                Transform::from_xyz(-45., 0., 0.)
            ),
            (
                WallShooter,
                Platoon(formation.id()),
                Transform::from_xyz(45., 0., 0.)
            ),
        ]);
    })
}

pub fn triple_wall() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (
                WallShooter,
                WallEmitter::from_bullets(3),
                Platoon(formation.id()),
                Transform::from_xyz(-40., 0., 0.)
            ),
            (
                WallShooter,
                WallEmitter::from_bullets(3),
                Platoon(formation.id()),
                Transform::from_xyz(0., 20., 0.)
            ),
            (
                WallShooter,
                WallEmitter::from_bullets(3),
                Platoon(formation.id()),
                Transform::from_xyz(40., 0., 0.)
            ),
        ]);
    })
}

pub fn orb_slinger() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![(OrbSlinger, Platoon(formation.id()))]);
    })
}

pub fn double_orb_slinger() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (
                OrbSlinger,
                Platoon(formation.id()),
                Transform::from_xyz(-40., 0., 0.)
            ),
            (
                OrbSlinger,
                Platoon(formation.id()),
                Transform::from_xyz(40., 0., 0.)
            ),
        ]);
    })
}

pub fn crisscross() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![(CrissCross, Platoon(formation.id()))]);
    })
}

pub fn double_crisscross() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (
                CrissCross,
                Platoon(formation.id()),
                Transform::from_xyz(-40., 0., 0.)
            ),
            (
                CrissCross,
                Platoon(formation.id()),
                Transform::from_xyz(40., 0., 0.)
            ),
        ]);
    })
}

const SWARM_OFFSET: f32 = crate::WIDTH / 1.2;
const NUM_SWARM: usize = 8;
const SWARM_GAP: f32 = 15.;

pub fn swarm_right() -> Formation {
    Formation::with_velocity(
        DEFAULT_FORMATION_VEL * 1.5,
        |formation: &mut EntityCommands, _| {
            formation.with_children(|root| {
                for i in 0..NUM_SWARM {
                    let x = (i as f32 - NUM_SWARM as f32 / 2.) * SWARM_GAP;
                    root.spawn((
                        Swarm,
                        SwarmMovement::Left,
                        Platoon(root.target_entity()),
                        EmitterDelay::new(0.2 * i as f32),
                        Transform::from_xyz(x + SWARM_OFFSET, 0., 0.),
                    ));
                }
            });
        },
    )
}

pub fn swarm_left() -> Formation {
    Formation::with_velocity(
        DEFAULT_FORMATION_VEL * 1.5,
        |formation: &mut EntityCommands, _| {
            formation.with_children(|root| {
                for i in 0..NUM_SWARM {
                    let x = (i as f32 - NUM_SWARM as f32 / 2.) * SWARM_GAP;
                    root.spawn((
                        Swarm,
                        SwarmMovement::Right,
                        Platoon(root.target_entity()),
                        EmitterDelay::new(0.2 * i as f32),
                        Transform::from_xyz(x - SWARM_OFFSET, 0., 0.),
                    ));
                }
            });
        },
    )
}

#[derive(Component)]
struct LaserEmitterTween {
    start: f32,
    end: f32,
}

impl Interpolator for LaserEmitterTween {
    type Item = LaserEmitter;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.dir = Vec2::from_angle(self.start.lerp(self.end, value));
    }
}

pub fn laser_maze() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            let laser = root
                .spawn((InvincibleLaserNode, LaserEmitter::new(Vec2::NEG_Y)))
                .id();
            root.commands()
                .entity(laser)
                .animation()
                .repeat(Repeat::Infinitely)
                .insert_tween_here(
                    Duration::from_secs_f32(10.),
                    EaseKind::Linear,
                    laser.into_target().with(LaserEmitterTween {
                        start: 0.,
                        end: PI * 2.,
                    }),
                );
        });
    })
}

pub fn laser_ladder() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            root.spawn((
                InvincibleLaserNode,
                LaserEmitter::new(Vec2::X),
                Transform::from_xyz(-35., 0., 0.),
            ));
            root.spawn((
                InvincibleLaserNode,
                LaserEmitter::new(Vec2::NEG_X),
                Transform::from_xyz(35., 50., 0.),
            ));
            root.spawn((
                InvincibleLaserNode,
                LaserEmitter::new(Vec2::X),
                Transform::from_xyz(-35., 100., 0.),
            ));
            root.spawn((
                InvincibleLaserNode,
                LaserEmitter::new(Vec2::NEG_X),
                Transform::from_xyz(35., 150., 0.),
            ));
        });
    })
}

pub fn boss() -> Formation {
    Formation::with_velocity(Vec2::ZERO, |formation: &mut EntityCommands, _| {
        formation.commands().remove_resource::<WaveTimeline>();
        formation.commands().spawn(gradius::Gradius);
    })
}

#[derive(Clone, Copy, Component)]
#[relationship(relationship_target = Units)]
#[component(on_remove = on_remove_platoon)]
pub struct Platoon(pub Entity);

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
pub struct Units(Vec<Entity>);

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

//fn spawn_formations(
//    mut commands: Commands,
//    server: Res<AssetServer>,
//    new_formations: Query<(Entity, &Formation), Added<Formation>>,
//    //formations: Query<(Entity, &Transform), (With<Formation>, With<Units>)>,
//) {
//    //let mut additional_height = 0.;
//    for (root, formation) in new_formations.iter() {
//        info!("spawn new formation");
//
//        //additional_height += formation.height() + PADDING;
//
//        let start_y = HEIGHT / 2. + LARGEST_SPRITE_SIZE / 2.;
//        //let end_y = HEIGHT / 2. + formation.topy() - LARGEST_SPRITE_SIZE / 2.;
//
//        let start = Vec3::new(0., start_y, ENEMY_Z);
//        //let end = Vec3::new(0., end_y - 20., 0.);
//
//        //commands.entity(root).animation().insert(tween(
//        //    Duration::from_secs_f32(FORMATION_EASE_DUR),
//        //    EaseKind::SineOut,
//        //    root.into_target().with(translation(start, end)),
//        //));
//
//        let mut commands = commands.entity(root);
//        (formation.spawn)(&mut commands, &server);
//        for mut modifier in formation.modifiers.iter_mut() {
//            modifier(&mut commands);
//        }
//        commands.insert(Transform::from_translation(start));
//    }
//
//    //if additional_height > 0. {
//    //    for (entity, transform) in formations.iter() {
//    //        commands.animation().insert(tween(
//    //            Duration::from_secs_f32(FORMATION_EASE_DUR),
//    //            EaseKind::SineOut,
//    //            entity.into_target().with(translation(
//    //                transform.translation,
//    //                Vec3::new(0., additional_height, 0.),
//    //            )),
//    //        ));
//    //    }
//    //}
//}

fn update_formations(
    time: Res<Time<Physics>>,
    mut formations: Query<(&mut Transform, &FormationEntity)>,
) {
    for (mut transform, formation) in formations.iter_mut() {
        transform.translation += formation.0.extend(0.) * time.delta_secs();
    }
}

//fn despawn_off_screen(
//    mut commands: Commands,
//    enemies: Query<(Entity, &GlobalTransform), With<EnemyType>>,
//) {
//    for (entity, transform) in enemies.iter() {
//        let p = transform.compute_transform().translation;
//        let w = crate::WIDTH / 2.;
//        let h = crate::HEIGHT / 2.;
//
//        if p.x < -w || p.y > w || p.y < -h || p.y > h {
//            commands.entity(entity).despawn();
//        }
//    }
//}

fn despawn_formations(
    mut commands: Commands,
    formations: Query<
        (Entity, &UnitDeaths, Option<&DropOption>),
        (With<FormationEntity>, Without<Units>),
    >,
    //formations: Query<(Entity, &UnitDeaths), (With<Formation>, Without<Units>)>,
    //off_screen: Query<(Entity, &Transform, &Formation)>,
) {
    //let mut rng = rand::rng();
    for (entity, deaths, drop) in formations.iter() {
        commands.entity(entity).despawn();
        if let Some(drop) = drop {
            commands.spawn((
                Pickup::Weapon(drop.0),
                drop.0,
                Transform::from_translation(deaths.last_death_position().unwrap().extend(1.)),
            ));
        }
        //if rng.random_bool(0.75) {
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

    //for (entity, transform, formation) in off_screen.iter() {
    //    let p = transform.translation;
    //    let w = crate::WIDTH / 2.;
    //    let h = crate::HEIGHT / 2. + formation.height() + 10.;
    //
    //    if p.x < -w || p.x > w || p.y < -h || p.y > h {
    //        info!("despawn formation off screen");
    //        commands
    //            .entity(entity)
    //            .despawn_related::<Children>()
    //            .despawn();
    //    }
    //}
}

fn animate_entrance(
    server: &AssetServer,
    commands: &mut Commands,
    bundle: impl Bundle + Clone,
    delay: Option<f32>,
    secs: f32,
    tstart: Vec3,
    tend: Vec3,
    rstart: Quat,
    rend: Quat,
) {
    match delay {
        Some(delay) => {
            run_after(
                Duration::from_secs_f32(delay),
                move |mut commands: Commands, server: Res<AssetServer>| {
                    let entity = commands.spawn(bundle.clone()).id();
                    animate_entrance_inner(
                        &server,
                        &mut commands,
                        entity,
                        secs,
                        tstart,
                        tend,
                        rstart,
                        rend,
                    );
                },
                commands,
            );
        }
        None => {
            let entity = commands.spawn(bundle).id();
            animate_entrance_inner(server, commands, entity, secs, tstart, tend, rstart, rend);
        }
    }
}

fn animate_entrance_inner(
    server: &AssetServer,
    commands: &mut Commands,
    entity: Entity,
    secs: f32,
    tstart: Vec3,
    tend: Vec3,
    rstart: Quat,
    rend: Quat,
) {
    commands.spawn((
        SamplePlayer::new(server.load("audio/sfx/rockets.wav")),
        PlaybackSettings {
            volume: Volume::Linear(0.3),
            ..PlaybackSettings::ONCE
        },
    ));

    let trail = commands
        .spawn((
            ParticleSpawner::default(),
            ParticleEffectHandle(server.load("particles/ship_fire.ron")),
            Transform::from_translation(Vec2::ZERO.extend(-100.))
                .with_rotation(Quat::from_rotation_z(PI)),
        ))
        .id();
    commands.entity(entity).add_child(trail);

    let id = entity;
    run_after(
        Duration::from_secs_f32((secs - 0.3).clamp(0., f32::MAX)),
        move |mut commands: Commands,
              mut states: Query<&mut ParticleSpawnerState>,
              server: Res<AssetServer>| {
            if let Ok(mut state) = states.get_mut(trail) {
                state.active = false;
            }

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/doot.wav")),
                PlaybackParams {
                    speed: 0.9,
                    ..Default::default()
                },
                PlaybackSettings {
                    volume: Volume::Linear(0.4),
                    ..PlaybackSettings::ONCE
                },
            ));

            let sprite = commands
                .spawn((
                    Sprite::from_image(server.load("crosshair.png")),
                    Transform::from_xyz(0., 0., 1.),
                ))
                .id();
            commands
                .entity(sprite)
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(0.5),
                    EaseKind::Linear,
                    sprite
                        .into_target()
                        .with(rotation(Quat::default(), Quat::from_rotation_z(PI))),
                )
                .animation()
                .insert_tween_here(
                    Duration::from_secs_f32(0.5),
                    EaseKind::QuadraticOut,
                    sprite
                        .into_target()
                        .with(sprite_color(Color::WHITE, Color::WHITE.with_alpha(0.))),
                );
            commands.entity(id).add_child(sprite);

            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/blurp.wav")),
                PlaybackSettings {
                    volume: Volume::Linear(0.15),
                    ..PlaybackSettings::ONCE
                },
            ));
        },
        commands,
    );

    commands
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(secs),
            EaseKind::QuadraticIn,
            id.into_target()
                .with(sprite_color(Color::srgb(0.2, 0.2, 0.2), WHITE.into())),
        )
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(secs),
            EaseKind::QuadraticOut,
            id.into_target().with(translation(tstart, tend)),
        )
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(secs),
            EaseKind::QuadraticOut,
            id.into_target().with(rotation(rstart, rend)),
        )
        .insert(ChildOf(id));
}
