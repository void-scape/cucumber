use crate::{
    assets,
    auto_collider::ImageCollider,
    bullet::{BulletRate, BulletSpeed, BulletTimer, Polarity, emitter::DualEmitter},
    health::{Dead, Health, HealthSet},
    pickups::{self, PickupEvent, Upgrade},
};
use bevy::{
    ecs::{component::ComponentId, system::RunSystemOnce, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use physics::{
    Physics,
    layers::{self, CollidesWith, TriggersWith},
    prelude::*,
};
use std::{cmp::Ordering, time::Duration};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut commands: Commands| {
            commands
                .spawn(Player)
                .with_child((DualEmitter::<layers::Enemy>::new(), Polarity::North));
        })
        .add_systems(Update, handle_pickups)
        .add_systems(Physics, handle_death.after(HealthSet))
        .add_input_context::<AliveContext>()
        .add_observer(apply_movement)
        .add_observer(stop_movement);
    }
}

#[derive(Component)]
#[require(
    Transform, Velocity, layers::Player, Health(|| Health::PLAYER),
    ImageCollider, BulletSpeed(|| BulletSpeed(1.0)), BulletRate(|| BulletRate(1.0)),
    TriggersWith::<pickups::PickupLayer>, CollidesWith::<layers::Wall>, DynamicBody
)]
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
                        assets::sprite_rect8(&server, assets::SHIPS_PATH, UVec2::new(1, 4)),
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
    mut player: Query<(&mut Velocity, &mut Sprite), With<Player>>,
) {
    let Ok((mut velocity, mut sprite)) = player.get_single_mut() else {
        return;
    };

    velocity.0 = trigger.value.normalize_or_zero() * 60.;
    if velocity.0.x.abs() < f32::EPSILON {
        velocity.0.x = 0.;
    }

    let (tl, br) = match velocity.0.x.total_cmp(&0.) {
        Ordering::Less => (Vec2::new(0., 4.), Vec2::new(1., 5.)),
        Ordering::Greater => (Vec2::new(2., 4.), Vec2::new(3., 5.)),
        Ordering::Equal => (Vec2::new(1., 4.), Vec2::new(2., 5.)),
    };

    sprite.rect = Some(Rect::from_corners(tl * 8., br * 8.));
}

fn stop_movement(
    _: Trigger<Completed<MoveAction>>,
    mut player: Query<(&mut Velocity, &mut Sprite), With<Player>>,
) {
    let Ok((mut velocity, mut sprite)) = player.get_single_mut() else {
        return;
    };

    velocity.0 = Vec2::default();
    let tl = Vec2::new(1., 4.) * 8.;
    let br = Vec2::new(2., 5.) * 8.;
    sprite.rect = Some(Rect::from_corners(tl, br));
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

#[derive(InputContext)]
struct AliveContext;

fn handle_death(q: Query<Entity, (With<Player>, With<Dead>)>, mut commands: Commands) {
    let Ok(player) = q.get_single() else {
        return;
    };

    info!("player died");

    commands.entity(player).despawn_recursive();
}

fn handle_pickups(
    mut q: Query<(&mut BulletSpeed, &mut BulletRate), With<Player>>,
    mut events: EventReader<PickupEvent>,
) {
    let Ok((mut speed, mut rate)) = q.get_single_mut() else {
        return;
    };

    for event in events.read() {
        match event {
            PickupEvent::Weapon(_) => {}
            PickupEvent::Upgrade(Upgrade::Speed(s)) => rate.0 += *s,
            _ => {}
        }
    }
}
