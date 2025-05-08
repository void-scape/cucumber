use crate::asteroids::AsteroidSpawner;
use crate::auto_collider::ImageCollider;
use crate::bullet::Destructable;
use crate::bullet::emitter::{BulletModifiers, HomingEmitter, LaserEmitter, SoloEmitter};
use crate::bullet::homing::TurnSpeed;
use crate::enemy::{CruiserExplosion, Enemy};
use crate::health::Dead;
use crate::player::{BlockControls, Player, WeaponEntity};
use crate::tween::OnEnd;
use crate::{GameState, Layer, end};
use crate::{
    HEIGHT,
    animation::{AnimationController, AnimationIndices},
    health::Health,
};
use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, RepeatStyle};
use bevy_tween::tween::IntoTarget;
use physics::linear_velocity;
use std::time::Duration;

const BOSS_HEALTH: f32 = 600.0;

pub struct MinibossPlugin;

impl Plugin for MinibossPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BossDeathEvent>()
            //.add_systems(OnEnter(GameState::StartGame), spawn_boss)
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(
                Update,
                (
                    update_boss_state,
                    update_solo_emitters,
                    handle_boss_death,
                    boss_death_effects,
                )
                    .chain()
                    .run_if(in_state(GameState::Game)),
            );
    }
}

fn restart(mut commands: Commands, boss: Single<Entity, With<Boss>>) {
    commands.entity(*boss).despawn();
}

#[derive(Component)]
#[require(Transform, Destructable)]
pub struct Boss;

const BOSS_EASE_DUR: f32 = 4.;

pub fn spawn_boss(mut commands: Commands) {
    let boss = commands
        .spawn((
            Boss,
            Enemy,
            DebugRect::from_size(Vec2::new(crate::WIDTH / 2., crate::WIDTH / 3.75)),
            ImageCollider,
            Health::full(BOSS_HEALTH),
            CollisionLayers::new(Layer::Enemy, Layer::Bullet),
            //Sprite {
            //    image: server.load("cruiser_base.png"),
            //    flip_y: true,
            //    ..Default::default()
            //},
            //Collider::rectangle(75., 90.),
            //CollisionTrigger(Collider::from_rect(
            //    Vec2::new(-25., 32.),
            //    Vec2::new(50., 64.),
            //)),
        ))
        .id();

    let start_y = HEIGHT / 2. + 64.;
    let end_y = start_y - 105.;
    let start = Vec3::ZERO.with_y(start_y);
    let end = Vec3::ZERO.with_y(end_y);

    commands
        .entity(boss)
        .insert(Transform::from_translation(end))
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(BOSS_EASE_DUR),
            EaseKind::SineOut,
            boss.into_target().with(translation(start, end)),
        );
}

fn update_boss_state(
    _enable: Single<&Boss>,
    mut commands: Commands,
    solo: Option<Single<Entity, With<InsertSoloDance>>>,
    wave: Option<Single<Entity, With<InsertWaves>>>,
    time: Res<Time<Physics>>,
    mut timer: Local<Option<Timer>>,
    mut last_wave: Local<(bool, usize)>,
) {
    let timer = timer.get_or_insert_with(|| {
        commands.spawn(InsertSoloDance::A);
        Timer::from_seconds(10., TimerMode::Repeating)
    });
    timer.tick(time.delta());
    if timer.just_finished() {
        if !last_wave.0 {
            last_wave.0 = true;
            if last_wave.1 > 0 {
                commands.spawn(InsertWaves::B);
            } else {
                commands.spawn(InsertWaves::A);
            }
            if let Some(solo) = solo {
                commands.entity(*solo).despawn();
            }
            last_wave.1 += 1;
        } else {
            last_wave.0 = false;
            commands.spawn(InsertSoloDance::B);
            if let Some(wave) = wave {
                commands.entity(*wave).despawn();
            }
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
enum SoloDance {
    A,
    B,
}

#[derive(Clone, Copy, Component)]
#[component(on_add = Self::guns, on_remove = Self::remove_guns)]
enum InsertSoloDance {
    A,
    B,
}

impl InsertSoloDance {
    fn guns(mut world: DeferredWorld, ctx: HookContext) {
        let solo = *world.get::<InsertSoloDance>(ctx.entity).unwrap();
        world.commands().queue(move |world: &mut World| {
            world.run_system_once(
                move |mut commands: Commands, boss: Single<Entity, With<Boss>>| {
                    let dance = match solo {
                        InsertSoloDance::A => SoloDance::A,
                        InsertSoloDance::B => SoloDance::B,
                    };

                    let dance = commands
                        .spawn((
                            dance,
                            BulletModifiers {
                                rate: 0.5,
                                speed: 0.8,
                                ..Default::default()
                            },
                            children![
                                ActiveSoloSet(SoloSet::A),
                                (LaserEmitter::player(), Transform::from_xyz(-35., -20., 0.)),
                                (LaserEmitter::player(), Transform::from_xyz(35., -20., 0.))
                            ],
                        ))
                        .id();
                    commands.entity(*boss).add_child(dance);
                },
            )
        });
    }

    fn remove_guns(mut world: DeferredWorld, _: HookContext) {
        world.commands().queue(|world: &mut World| {
            world
                .run_system_once(
                    |mut commands: Commands, dance: Single<Entity, With<SoloDance>>| {
                        commands.entity(*dance).despawn();
                    },
                )
                .unwrap();
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Component)]
enum SoloSet {
    A,
    B,
}

#[derive(Component)]
struct ActiveSoloSet(SoloSet);

fn update_solo_emitters(
    mut commands: Commands,
    dance: Single<(Entity, &SoloDance)>,
    mut active_solo_set: Single<&mut ActiveSoloSet>,
    solos: Query<Entity, With<SoloSet>>,
    mut solo_timer: Local<Option<Timer>>,
    time: Res<Time<Physics>>,
    mut phase_b_count: Local<usize>,
) {
    let (dance_entity, dance) = dance.into_inner();

    if active_solo_set.is_added() {
        match dance {
            SoloDance::A => {
                *solo_timer = Some(Timer::from_seconds(5., TimerMode::Repeating));
            }
            SoloDance::B => {
                *solo_timer = Some(Timer::from_seconds(
                    (1. - *phase_b_count as f32 * 0.2).max(0.4),
                    TimerMode::Repeating,
                ));
                *phase_b_count += 1;
            }
        }
    }

    let solo_timer = solo_timer.as_mut().unwrap();
    solo_timer.tick(time.delta());
    if solo_timer.just_finished() {
        match active_solo_set.0 {
            SoloSet::A => {
                active_solo_set.0 = SoloSet::B;
            }
            SoloSet::B => {
                active_solo_set.0 = SoloSet::A;
            }
        }
    }

    if active_solo_set.is_added() || active_solo_set.is_changed() {
        for entity in solos.iter() {
            commands.entity(entity).despawn();
        }

        match active_solo_set.0 {
            SoloSet::A => {
                commands.entity(dance_entity).with_children(|root| {
                    for p in [-25., -15., -5.].into_iter() {
                        root.spawn((
                            SoloEmitter::player(),
                            Transform::from_xyz(p, -20., 0.),
                            SoloSet::A,
                        ));
                    }
                });
            }
            SoloSet::B => {
                commands.entity(dance_entity).with_children(|root| {
                    for p in [25., 15., 5.].into_iter() {
                        root.spawn((
                            SoloEmitter::player(),
                            Transform::from_xyz(p, -20., 0.),
                            SoloSet::B,
                        ));
                    }
                });
            }
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Waves;

#[derive(Clone, Copy, Component)]
#[component(on_add = Self::guns, on_remove = Self::remove_guns)]
enum InsertWaves {
    A,
    B,
}

impl InsertWaves {
    fn guns(mut world: DeferredWorld, ctx: HookContext) {
        let wave = *world.get::<InsertWaves>(ctx.entity).unwrap();
        world.commands().queue(move |world: &mut World| {
            world.run_system_once(
                move |mut commands: Commands, boss: Single<Entity, With<Boss>>| {
                    let waves = commands
                        .spawn((
                            Waves,
                            BulletModifiers {
                                rate: 0.5,
                                speed: 0.8,
                                ..Default::default()
                            },
                        ))
                        .id();
                    match wave {
                        InsertWaves::A => {}
                        InsertWaves::B => {
                            commands.entity(waves).with_children(|root| {
                                root.spawn((
                                    HomingEmitter::<Player>::player(),
                                    TurnSpeed(60.),
                                    BulletModifiers {
                                        rate: 0.35,
                                        speed: 0.8,
                                        ..Default::default()
                                    },
                                    Transform::from_xyz(-45., -20., 0.),
                                ));
                                root.spawn((
                                    HomingEmitter::<Player>::player(),
                                    TurnSpeed(60.),
                                    BulletModifiers {
                                        rate: 0.35,
                                        speed: 0.8,
                                        ..Default::default()
                                    },
                                    Transform::from_xyz(45., -20., 0.),
                                ));
                            });
                        }
                    }
                    commands.entity(*boss).add_child(waves);

                    let wave = commands
                        .spawn((
                            Transform::default(),
                            Visibility::default(),
                            BulletModifiers {
                                rate: 0.6,
                                speed: 0.5,
                                ..Default::default()
                            },
                            children![
                                (SoloEmitter::player(), Transform::from_xyz(10., 0., 0.)),
                                (SoloEmitter::player(), Transform::from_xyz(0., 0., 0.)),
                                (SoloEmitter::player(), Transform::from_xyz(-10., 0., 0.)),
                            ],
                        ))
                        .id();
                    commands
                        .entity(waves)
                        .add_child(wave)
                        .animation()
                        .repeat_style(RepeatStyle::PingPong)
                        .insert_tween_here(
                            Duration::from_secs_f32(3.),
                            EaseKind::CubicInOut,
                            wave.into_target().with(translation(
                                Vec3::new(-45., -20., 0.),
                                Vec3::new(45., -20., 0.),
                            )),
                        );
                },
            )
        });
    }

    fn remove_guns(mut world: DeferredWorld, _: HookContext) {
        world.commands().queue(|world: &mut World| {
            world
                .run_system_once(
                    |mut commands: Commands, wave: Single<Entity, With<Waves>>| {
                        commands.entity(*wave).despawn();
                    },
                )
                .unwrap();
        });
    }
}

#[derive(Event)]
pub struct BossDeathEvent(Vec2);

fn handle_boss_death(
    boss: Single<(Entity, &GlobalTransform), (With<Dead>, With<Boss>)>,
    mut commands: Commands,
    mut writer: EventWriter<BossDeathEvent>,
    mut asteroids: ResMut<AsteroidSpawner>,
) {
    asteroids.0 = true;
    let (entity, transform) = boss.into_inner();
    writer.write(BossDeathEvent(
        transform.compute_transform().translation.xy(),
    ));
    commands.entity(entity).despawn();
}

fn boss_death_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    layout: Res<CruiserExplosion>,
    mut reader: EventReader<BossDeathEvent>,
    player: Single<(Entity, &WeaponEntity)>,
) {
    let (player, weapon) = player.into_inner();
    for event in reader.read() {
        commands.spawn((
            Sprite {
                image: server.load("cruiser_explosion.png"),
                texture_atlas: Some(TextureAtlas {
                    layout: layout.0.clone(),
                    index: 0,
                }),
                flip_y: true,
                ..Default::default()
            },
            AnimationController::from_seconds(AnimationIndices::once_despawn(0..=13), 0.1),
            Transform::from_translation(event.0.extend(1.)),
        ));

        commands.entity(weapon.0).despawn();
        commands.entity(player).remove::<WeaponEntity>();

        run_after(
            Duration::from_secs_f32(2.),
            |mut commands: Commands, player: Single<(Entity, &Transform), With<Player>>| {
                let (player, position) = player.into_inner();
                commands
                    .entity(player)
                    .insert((BlockControls, ColliderDisabled));

                let start = position.translation;
                let end = position.translation.with_y(crate::HEIGHT / 2. + 16.);
                let dist = end.y - position.translation.y;

                let on_end = OnEnd::new(&mut commands, end::show_win_screen);
                commands
                    .animation()
                    .insert_tween_here(
                        Duration::from_secs_f32(dist / crate::HEIGHT * 2.),
                        EaseKind::ExponentialIn,
                        player.into_target().with(translation(start, end)),
                    )
                    .insert(on_end);
                // keep the velocity above 0 so that we get blasters
                commands.animation().insert_tween_here(
                    Duration::from_secs_f32(dist / crate::HEIGHT * 2. + 1.),
                    EaseKind::Linear,
                    player
                        .into_target()
                        .with(linear_velocity(Vec2::Y, Vec2::ZERO)),
                );
            },
            &mut commands,
        );
    }
}
