use crate::asteroids::SpawnCluster;
use crate::auto_collider::ImageCollider;
use crate::bullet::Destructable;
use crate::bullet::emitter::{
    BuckShotEmitter, BulletModifiers, EmitterDelay, GradiusSpiralEmitter, PulseTime, Rate,
    SpiralOrbEmitter, WallEmitter,
};
use crate::enemy::Enemy;
use crate::health::{Dead, Health};
use crate::{DespawnRestart, GameState, Layer, RESOLUTION_SCALE};
use avian2d::prelude::CollisionLayers;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

const HEALTH: f32 = 500.;

pub struct GradiusPlugin;

impl Plugin for GradiusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_phase, phasea, update_health_display, kill_boss)
                .run_if(in_state(GameState::Game)),
        )
        .add_observer(init_gradius)
        .add_observer(enter_phasea)
        .add_observer(exit_phasea)
        .add_observer(enter_phaseb);
    }
}

#[derive(Component)]
#[require(
    Enemy,
    PhaseA,
    ImageCollider,
    Destructable,
    Health::full(HEALTH),
    Transform::from_xyz(0., crate::HEIGHT / 3., 0.),
    CollisionLayers::new(Layer::Enemy, Layer::Bullet),
    DespawnRestart,
)]
pub struct Gradius;

fn init_gradius(
    trigger: Trigger<OnAdd, Gradius>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    commands
        .entity(trigger.target())
        .insert(DebugRect::from_size(Vec2::new(
            crate::WIDTH / 2.,
            crate::WIDTH / 3.75,
        )));
    commands.spawn((
        HealthDisplay,
        HIGH_RES_LAYER,
        Text2d::default(),
        TextFont {
            font_size: 16.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_translation(
            (Vec2::new(0., crate::HEIGHT / 2. - 10.) * RESOLUTION_SCALE).extend(500.),
        ),
    ));
}

#[derive(Default, Component)]
struct PhaseA;

#[derive(Default, Component)]
pub struct PhaseB;

#[derive(Component)]
struct HealthDisplay;

#[derive(Component)]
struct FlipOrbEmitters(Timer);

impl FlipOrbEmitters {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Repeating))
    }
}

fn update_phase(
    mut commands: Commands,
    gradius: Single<(Entity, &Health), (With<Gradius>, With<PhaseA>)>,
) {
    let (entity, health) = gradius.into_inner();
    if health.current() <= health.max() / 2. {
        commands.entity(entity).remove::<PhaseA>().insert(PhaseB);
    }
}

fn enter_phasea(trigger: Trigger<OnAdd, PhaseA>, mut commands: Commands) {
    let orb = SpiralOrbEmitter::new(8, 2.0, 0.2);
    let total_time = orb.total_time();
    let buck_waves = 4;
    let buck_shot_dur = 0.2;
    let buck_wait = total_time - buck_shot_dur * buck_waves as f32;

    commands
        .entity(trigger.target())
        .insert(FlipOrbEmitters::new(orb.total_time()))
        .with_children(|root| {
            root.spawn((orb.clone(), Transform::from_xyz(-40., 0., 0.)));
            root.spawn((orb, Transform::from_xyz(40., 40., 0.)));

            root.spawn((
                BuckShotEmitter::new(buck_waves, buck_wait, buck_shot_dur),
                BulletModifiers {
                    speed: 0.5,
                    ..Default::default()
                },
                EmitterDelay::new(total_time / 2.),
            ));

            root.spawn((
                WallEmitter::from_dir(Vec2::from_angle(std::f32::consts::PI * 2. * 0.80)),
                BulletModifiers {
                    speed: 1.25,
                    rate: Rate::Secs(total_time),
                    ..Default::default()
                },
                EmitterDelay::new(total_time / 2.),
                Transform::from_xyz(-40., 30., 0.),
            ));
            root.spawn((
                WallEmitter::from_dir(Vec2::from_angle(std::f32::consts::PI * 2. * 0.70)),
                BulletModifiers {
                    speed: 1.25,
                    rate: Rate::Secs(total_time),
                    ..Default::default()
                },
                EmitterDelay::new(total_time / 2.),
                Transform::from_xyz(40., 30., 0.),
            ));
        });
}

fn phasea(
    time: Res<Time>,
    gradius: Single<(&Children, &mut FlipOrbEmitters), With<PhaseA>>,
    mut emitters: Query<&mut Transform, With<SpiralOrbEmitter>>,
) {
    let (children, mut flip_orbs) = gradius.into_inner();

    flip_orbs.0.tick(time.delta());
    if flip_orbs.0.just_finished() {
        let mut iter = emitters.iter_many_mut(children.iter());
        while let Some(mut transform) = iter.fetch_next() {
            match transform.translation.y {
                0. => transform.translation.y = 40.,
                40. => transform.translation.y = 0.,
                _ => unreachable!(),
            }
        }
    }
}

fn exit_phasea(trigger: Trigger<OnRemove, PhaseA>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .try_remove::<FlipOrbEmitters>()
        .despawn_related::<Children>();
}

fn enter_phaseb(trigger: Trigger<OnAdd, PhaseB>, mut commands: Commands) {
    commands.entity(trigger.target()).with_children(|root| {
        root.spawn(GradiusSpiralEmitter);
        root.spawn((
            WallEmitter::new(Vec2::NEG_Y, 15, 20.),
            BulletModifiers {
                rate: Rate::Secs(2.),
                ..Default::default()
            },
        ));
        root.spawn((
            WallEmitter::new(Vec2::NEG_Y, 15, 20.),
            BulletModifiers {
                rate: Rate::Secs(2.),
                ..Default::default()
            },
            EmitterDelay::new(1.),
            Transform::from_xyz(10., 0., 0.),
        ));
    });
}

fn update_health_display(
    health: Single<&Health, (With<Gradius>, Changed<Health>)>,
    mut text: Single<&mut Text2d, With<HealthDisplay>>,
) {
    text.0 = format!("Health: {:.2} / {:.2}", health.current(), health.max());
}

fn kill_boss(
    mut commands: Commands,
    boss: Single<(Entity, &Transform), (With<Gradius>, With<Dead>)>,
    health: Single<Entity, With<HealthDisplay>>,
    mut writer: EventWriter<SpawnCluster>,
) {
    let (entity, transform) = boss.into_inner();
    commands.entity(entity).despawn();
    commands.entity(*health).despawn();

    let position = transform.translation.xy();
    writer.write(SpawnCluster {
        parts: 10,
        shield: 10,
        position,
    });
    writer.write(SpawnCluster {
        parts: 10,
        shield: 10,
        position: position + Vec2::new(-40., 40.),
    });
    writer.write(SpawnCluster {
        parts: 10,
        shield: 10,
        position: position + Vec2::new(40., 40.),
    });
}
