use crate::asteroids::AsteroidSpawner;
use crate::auto_collider::ImageCollider;
use crate::bullet::Destructable;
use crate::bullet::emitter::{BaseSpeed, HomingEmitter, LaserEmitter, SoloEmitter};
use crate::bullet::homing::TurnSpeed;
use crate::enemy::CruiserExplosion;
use crate::health::Dead;
use crate::player::Player;
use crate::{GameState, Layer};
use crate::{
    HEIGHT,
    animation::{AnimationController, AnimationIndices},
    bullet::{BulletRate, BulletSpeed},
    health::Health,
};
use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity_disabling::Disabled;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;
use bevy_seedling::prelude::*;
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, RepeatStyle};
use bevy_tween::tween::IntoTarget;
use std::time::Duration;

pub struct MinibossPlugin;

impl Plugin for MinibossPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BossDeathEvent>()
            .add_systems(OnEnter(GameState::StartGame), spawn_boss)
            .add_systems(
                Update,
                (
                    update_boss_state,
                    update_solo_emitters,
                    boss_death_effects,
                    handle_boss_death,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Component)]
#[require(Transform, Destructable)]
struct Boss;

//const BOSS_EASE_DUR: f32 = 4.;

pub fn spawn_boss(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut asteroids: ResMut<AsteroidSpawner>,
) {
    commands.spawn((
        SamplePlayer::new(server.load("audio/music/midir.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::Linear(0.5),
        }],
    ));

    asteroids.0 = false;
    let boss = commands
        .spawn((
            Boss,
            DebugRect::from_size(Vec2::new(crate::WIDTH / 2., crate::WIDTH / 3.75)),
            ImageCollider,
            Health::full(500.0),
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
    //let start = Vec3::ZERO.with_y(start_y);
    let end = Vec3::ZERO.with_y(end_y);

    //let init = AttackPattern::SoloDance
    //    .on_start(init_solo_emitter)
    //    .on_visit(update_solo_emitters)
    //    .on_end(clean_solo_emitters)
    //    .always()
    //    .once();
    //spawn_root(init, &mut commands);

    commands
        .entity(boss)
        .insert(Transform::from_translation(end));
    //commands.animation().insert(tween(
    //    Duration::from_secs_f32(BOSS_EASE_DUR),
    //    EaseKind::SineOut,
    //    boss.into_target().with(translation(start, end)),
    //));
}

fn update_boss_state(
    mut commands: Commands,
    solo: Option<Single<Entity, With<InsertSoloDance>>>,
    wave: Option<Single<Entity, With<InsertWaves>>>,
    time: Res<Time<Physics>>,
    mut timer: Local<Option<Timer>>,
    mut last_wave: Local<(bool, usize)>,
) {
    let timer = timer.get_or_insert_with(|| {
        //commands.spawn(InsertWaves);
        //*last_wave = true;

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
struct SoloDance;

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
                    let dance = commands
                        .spawn((
                            SoloDance,
                            BulletRate(0.5),
                            BulletSpeed(0.8),
                            children![
                                ActiveSoloSet(SoloSet::A),
                                //
                                (
                                    SoloEmitter::player(),
                                    Transform::from_xyz(-25., -20., 0.),
                                    SoloSet::A,
                                ),
                                (
                                    SoloEmitter::player(),
                                    Transform::from_xyz(-15., -20., 0.),
                                    SoloSet::A,
                                ),
                                (
                                    SoloEmitter::player(),
                                    Transform::from_xyz(-5., -20., 0.),
                                    SoloSet::A,
                                ),
                                //
                                (
                                    SoloEmitter::player(),
                                    Transform::from_xyz(25., -20., 0.),
                                    SoloSet::B,
                                ),
                                (
                                    SoloEmitter::player(),
                                    Transform::from_xyz(15., -20., 0.),
                                    SoloSet::B,
                                ),
                                (
                                    SoloEmitter::player(),
                                    Transform::from_xyz(5., -20., 0.),
                                    SoloSet::B,
                                ),
                            ],
                        ))
                        .id();
                    commands.entity(*boss).add_child(dance);

                    match solo {
                        InsertSoloDance::A => {
                            commands.entity(dance).with_children(|root| {
                                root.spawn((
                                    LaserEmitter::player(),
                                    Transform::from_xyz(-35., -20., 0.),
                                ));
                                root.spawn((
                                    LaserEmitter::player(),
                                    Transform::from_xyz(35., -20., 0.),
                                ));
                                root.spawn((
                                    HomingEmitter::<Player>::player(),
                                    TurnSpeed(60.),
                                    BaseSpeed(100.),
                                    BulletRate(0.35),
                                    Transform::from_xyz(-45., -20., 0.),
                                ));
                                root.spawn((
                                    HomingEmitter::<Player>::player(),
                                    TurnSpeed(60.),
                                    BaseSpeed(100.),
                                    BulletRate(0.35),
                                    Transform::from_xyz(45., -20., 0.),
                                ));
                            });
                        }
                        InsertSoloDance::B => {
                            let lasers = commands
                                .spawn((Transform::default(), Visibility::default()))
                                .with_children(|root| {
                                    root.spawn((
                                        LaserEmitter::player(),
                                        Transform::from_xyz(-35., 0., 0.),
                                    ));
                                    root.spawn((
                                        LaserEmitter::player(),
                                        Transform::from_xyz(35., 0., 0.),
                                    ));
                                })
                                .id();
                            commands
                                .entity(dance)
                                .add_child(lasers)
                                .animation()
                                .repeat_style(RepeatStyle::PingPong)
                                .insert_tween_here(
                                    Duration::from_secs_f32(10.),
                                    EaseKind::Linear,
                                    lasers.into_target().with(translation(
                                        Vec3::new(-25., -20., 0.),
                                        Vec3::new(25., -20., 0.),
                                    )),
                                );
                        }
                    }
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
    mut active_solo_set: Single<&mut ActiveSoloSet>,
    solos: Query<(Entity, &SoloSet), Or<(With<Disabled>, Without<Disabled>)>>,
    mut solo_timer: Local<Option<Timer>>,
    time: Res<Time<Physics>>,
) {
    if active_solo_set.is_added() {
        *solo_timer = Some(Timer::from_seconds(5., TimerMode::Repeating));
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
        for (entity, solo) in solos.iter() {
            if *solo == active_solo_set.0 {
                commands.entity(entity).remove::<Disabled>();
            } else {
                commands.entity(entity).insert(Disabled);
            }
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct Waves;

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
                        .spawn((Waves, BulletRate(0.5), BulletSpeed(0.8)))
                        .id();
                    match wave {
                        InsertWaves::A => {}
                        InsertWaves::B => {
                            commands.entity(waves).with_children(|root| {
                                root.spawn((
                                    HomingEmitter::<Player>::player(),
                                    TurnSpeed(60.),
                                    BaseSpeed(100.),
                                    BulletRate(0.35),
                                    Transform::from_xyz(-45., -20., 0.),
                                ));
                                root.spawn((
                                    HomingEmitter::<Player>::player(),
                                    TurnSpeed(60.),
                                    BaseSpeed(100.),
                                    BulletRate(0.35),
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
                            BulletRate(0.6),
                            BulletSpeed(0.5),
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
struct BossDeathEvent(Vec2);

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
) {
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
    }
}
