use std::time::Duration;

use bevy::{
    ecs::{component::ComponentId, system::RunSystemOnce, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use physics::{
    layers::{self, CollidesWith},
    prelude::*,
};

use crate::bullet::{BulletTimer, BulletType};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Player);
        })
        .add_systems(Update, shoot_bullets)
        .add_input_context::<AliveContext>()
        .add_observer(apply_movement)
        .add_observer(stop_movement);
    }
}

#[derive(Component)]
#[require(Transform, Velocity)]
#[component(on_add = Self::on_add)]
pub struct Player;

impl Player {
    fn on_add(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        world.commands().queue(move |world: &mut World| {
            world
                .run_system_once(move |mut commands: Commands, server: Res<AssetServer>| {
                    let mut actions = Actions::<AliveContext>::default();
                    actions.bind::<MoveAction>().to((
                        Cardinal::wasd_keys(),
                        Cardinal::arrow_keys(),
                        Cardinal::dpad_buttons(),
                        GamepadStick::Left.with_modifiers_each(
                            DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15),
                        ),
                    ));

                    commands.entity(entity).insert((
                        actions,
                        Sprite {
                            image: server.load("invaders_sprites.png"),
                            rect: Some(Rect::from_corners(Vec2::ZERO, Vec2::ONE * 8.)),
                            ..Default::default()
                        },
                        BulletTimer {
                            timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
                        },
                    ));
                })
                .unwrap();
        });
    }
}

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    player: Single<&mut Velocity, With<Player>>,
) {
    let mut velocity = player.into_inner();
    velocity.0 = trigger.value.normalize_or_zero() * 200.;
}

fn stop_movement(_: Trigger<Completed<MoveAction>>, player: Single<&mut Velocity, With<Player>>) {
    let mut velocity = player.into_inner();
    velocity.0 = Vec2::default();
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

#[derive(InputContext)]
struct AliveContext;

fn shoot_bullets(
    mut player: Query<(&mut BulletTimer, &Transform), With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let Ok((mut timer, transform)) = player.get_single_mut() else {
        return;
    };

    timer.timer.tick(time.delta());

    if timer.timer.just_finished() {
        let mut new_transform = transform.clone();
        new_transform.translation.y += 5.0;

        commands.spawn((
            BulletType::Basic,
            Velocity(Vec2::new(0.0, 300.0)),
            new_transform,
            CollidesWith::<layers::Enemy>::default(),
        ));
    }
}
