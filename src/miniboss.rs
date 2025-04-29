use crate::GameState;
use crate::bullet::homing::TurnSpeed;
use crate::enemy::CruiserExplosion;
use crate::health::{Dead, HealthSet};
use crate::{
    HEIGHT,
    animation::{AnimationController, AnimationIndices},
    bullet::{
        BulletRate, BulletSpeed,
        emitter::{DualEmitter, HomingEmitter},
    },
    health::Health,
    player::Player,
};
use bevy::prelude::*;
use bevy_tween::{
    combinator::tween,
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use physics::prelude::*;
use std::time::Duration;

pub struct MinibossPlugin;

impl Plugin for MinibossPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BossDeathEvent>()
            .add_systems(OnEnter(GameState::Game), spawn_boss)
            .add_systems(Update, boss_death_effects.run_if(in_state(GameState::Game)))
            .add_systems(Physics, handle_boss_death.after(HealthSet));
    }
}

#[derive(Component)]
#[require(Transform, layers::Enemy)]
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
            Health::full(100),
            BulletRate(0.5),
            BulletSpeed(0.8),
            CollisionTrigger(Collider::from_rect(
                Vec2::new(-25., 32.),
                Vec2::new(50., 64.),
            )),
        ))
        .with_children(|root| {
            //root.spawn((
            //    DualEmitter::<layers::Player>::new(2.),
            //    Transform::from_xyz(-19., -20., 0.),
            //));
            //root.spawn((
            //    DualEmitter::<layers::Player>::new(2.),
            //    Transform::from_xyz(19., -20., 0.),
            //));

            //root.spawn((
            //    HomingEmitter::<layers::Player, Player>::new(),
            //    TurnSpeed(60.),
            //    Transform::from_xyz(30., 10., 0.),
            //));
            //root.spawn((
            //    HomingEmitter::<layers::Player, Player>::new(),
            //    TurnSpeed(60.),
            //    Transform::from_xyz(-30., 10., 0.),
            //));
        })
        .id();

    let start_y = HEIGHT / 2. + 64.;
    let end_y = start_y - 105.;
    let start = Vec3::ZERO.with_y(start_y);
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
    q: Query<(Entity, &GlobalTransform), (With<Dead>, With<Boss>)>,
    mut commands: Commands,
    mut writer: EventWriter<BossDeathEvent>,
) {
    for (entity, transform) in q.iter() {
        writer.write(BossDeathEvent(
            transform.compute_transform().translation.xy(),
        ));
        commands.entity(entity).despawn();
    }
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
