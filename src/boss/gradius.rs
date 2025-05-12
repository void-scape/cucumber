use crate::asteroids::SpawnCluster;
use crate::bullet::Destructable;
use crate::bullet::emitter::{BuckShotEmitter, BulletModifiers, EmitterDelay, OrbEmitter};
use crate::enemy::Enemy;
use crate::health::{Dead, Health};
use crate::{DespawnRestart, Layer};
use avian2d::prelude::CollisionLayers;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

const HEALTH: f32 = 500.;

pub struct GradiusPlugin;

impl Plugin for GradiusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (phasea, update_health_display, kill_boss))
            .add_observer(enter_phasea);
    }
}

#[derive(Component)]
pub struct Gradius;

#[derive(Component)]
struct PhaseA;

#[derive(Component)]
struct HealthDisplay;

pub fn gradius(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        Gradius,
        Enemy,
        PhaseA,
        Destructable,
        Health::full(HEALTH),
        DebugRect::from_size(Vec2::new(crate::WIDTH / 2., crate::WIDTH / 3.75)),
        CollisionLayers::new(Layer::Enemy, Layer::Bullet),
        Transform::from_xyz(0., crate::HEIGHT / 3., 0.),
        DespawnRestart,
        children![(
            HealthDisplay,
            HIGH_RES_LAYER,
            Text2d("Choose Upgrade".into()),
            TextFont {
                font_size: 16.,
                font: server.load("fonts/joystix.otf"),
                ..Default::default()
            },
            Transform::from_xyz(0., 250., 500.),
        )],
    ));
}

#[derive(Component)]
struct FlipOrbEmitters(Timer);

impl FlipOrbEmitters {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Repeating))
    }
}

fn enter_phasea(trigger: Trigger<OnAdd, PhaseA>, mut commands: Commands) {
    let orb = OrbEmitter::player().with_all(8, 2.0, 0.2);
    let buck_waves = 4;
    let buck_shot_dur = 0.2;
    let buck_wait = orb.total_time() - buck_shot_dur * buck_waves as f32;

    commands
        .entity(trigger.target())
        .insert(FlipOrbEmitters::new(orb.total_time()))
        .with_children(|root| {
            root.spawn((orb.clone(), Transform::from_xyz(-40., 0., 0.)));
            root.spawn((orb, Transform::from_xyz(40., 40., 0.)));
            root.spawn((
                BuckShotEmitter::player().with_all(buck_waves, buck_wait, buck_shot_dur),
                EmitterDelay::new(orb.total_time() / 2.),
                BulletModifiers {
                    speed: 0.5,
                    ..Default::default()
                },
            ));
        });
}

fn phasea(
    time: Res<Time>,
    gradius: Single<(&Children, &mut FlipOrbEmitters), With<PhaseA>>,
    mut emitters: Query<&mut Transform, With<OrbEmitter>>,
) {
    let (children, mut flip_orbs) = gradius.into_inner();

    flip_orbs.0.tick(time.delta());
    if flip_orbs.0.just_finished() {
        info!("flip");
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

fn update_health_display(
    health: Single<&Health, (With<Gradius>, Changed<Health>)>,
    mut text: Single<&mut Text2d, With<HealthDisplay>>,
) {
    text.0 = format!("Health: {:.2} / {:.2}", health.current(), health.max());
}

fn kill_boss(
    mut commands: Commands,
    boss: Single<(Entity, &Transform), (With<Gradius>, With<Dead>)>,
    mut writer: EventWriter<SpawnCluster>,
) {
    let (entity, transform) = boss.into_inner();
    commands.entity(entity).despawn();

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
