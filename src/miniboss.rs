use crate::bullet::Destructable;
use crate::enemy::CruiserExplosion;
use crate::health::Dead;
use crate::{GameState, Layer};
use crate::{
    HEIGHT,
    animation::{AnimationController, AnimationIndices},
    bullet::{BulletRate, BulletSpeed},
    health::Health,
};
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct MinibossPlugin;

impl Plugin for MinibossPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BossDeathEvent>()
            //.add_systems(OnEnter(GameState::Game), spawn_boss)
            .add_systems(
                Update,
                (boss_death_effects, handle_boss_death).run_if(in_state(GameState::Game)),
            );
    }
}

#[derive(Component)]
#[require(Transform, Destructable, Sensor)]
struct Boss;

const BOSS_EASE_DUR: f32 = 4.;

pub fn spawn_boss(mut commands: Commands, server: Res<AssetServer>) {
    info!("spawning miniboss");
    let boss = commands
        .spawn((
            Boss,
            Sprite {
                image: server.load("cruiser_base.png"),
                flip_y: true,
                ..Default::default()
            },
            Health::full(100.0),
            BulletRate(0.5),
            BulletSpeed(0.8),
            Collider::rectangle(75., 90.),
            CollisionLayers::new(Layer::Enemy, 0),
            //CollisionTrigger(Collider::from_rect(
            //    Vec2::new(-25., 32.),
            //    Vec2::new(50., 64.),
            //)),

            //children![(
            //    DualEmitter::<layers::Player>::new(2.),
            //    Transform::from_xyz(-19., -20., 0.),
            //)(
            //    DualEmitter::<layers::Player>::new(2.),
            //    Transform::from_xyz(19., -20., 0.),
            //)(
            //    HomingEmitter::<layers::Player, Player>::new(),
            //    TurnSpeed(60.),
            //    Transform::from_xyz(30., 10., 0.),
            //)(
            //    HomingEmitter::<layers::Player, Player>::new(),
            //    TurnSpeed(60.),
            //    Transform::from_xyz(-30., 10., 0.),
            //)],
        ))
        .id();

    let start_y = HEIGHT / 2. + 64.;
    let end_y = start_y - 105.;
    //let start = Vec3::ZERO.with_y(start_y);
    let end = Vec3::ZERO.with_y(end_y);

    commands
        .entity(boss)
        .insert(Transform::from_translation(end));
    //commands.animation().insert(tween(
    //    Duration::from_secs_f32(BOSS_EASE_DUR),
    //    EaseKind::SineOut,
    //    boss.into_target().with(translation(start, end)),
    //));
}

#[derive(Event)]
struct BossDeathEvent(Vec2);

fn handle_boss_death(
    boss: Single<(Entity, &GlobalTransform), (With<Dead>, With<Boss>)>,
    mut commands: Commands,
    mut writer: EventWriter<BossDeathEvent>,
) {
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
