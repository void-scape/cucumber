use crate::{
    animation::{AnimationController, AnimationIndices, AnimationMode},
    assets::{self, MISC_PATH, MiscLayout},
    auto_collider::ImageCollider,
    bullet::emitter::DualEmitter,
    bullet::{BulletRate, BulletSpeed, BulletTimer, Polarity, emitter::HomingEmitter},
    enemy::Enemy,
    health::{Dead, Health, HealthSet},
    pickups::{self, PickupEvent, Upgrade, Weapon},
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
use std::{cmp::Ordering, f32, time::Duration};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut commands: Commands| {
            let starting_weapon = commands
                .spawn((DualEmitter::<layers::Enemy>::new(), Polarity::North))
                .id();

            commands
                .spawn((Player, WeaponEntity(starting_weapon)))
                .add_child(starting_weapon);
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

#[derive(Component)]
struct WeaponEntity(Entity);

impl Player {
    fn on_add(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        world.commands().queue(move |world: &mut World| {
            world
                .run_system_once(
                    move |mut commands: Commands,
                          server: Res<AssetServer>,
                          misc_layout: Res<MiscLayout>| {
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

                        commands.entity(entity).with_child((
                            PlayerBlasters,
                            Visibility::Hidden,
                            Transform::from_xyz(0., -7., -1.),
                            Sprite::from_atlas_image(
                                server.load(MISC_PATH),
                                TextureAtlas::from(misc_layout.0.clone()),
                            ),
                            AnimationController::from_seconds(
                                AnimationIndices::new(AnimationMode::Repeat, 18..=21),
                                0.1,
                            ),
                        ));
                    },
                )
                .unwrap();
        });
    }
}

// TODO: make this for enemies too?
#[derive(Component)]
struct PlayerBlasters;

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    mut player: Query<(&mut Velocity, &mut Sprite), With<Player>>,
    mut blasters: Query<&mut Visibility, With<PlayerBlasters>>,
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

    if let Ok(mut vis) = blasters.get_single_mut() {
        if velocity.0.y > f32::EPSILON {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn stop_movement(
    _: Trigger<Completed<MoveAction>>,
    mut player: Query<(&mut Velocity, &mut Sprite), With<Player>>,
    mut blasters: Query<&mut Visibility, With<PlayerBlasters>>,
) {
    let Ok((mut velocity, mut sprite)) = player.get_single_mut() else {
        return;
    };

    velocity.0 = Vec2::default();
    let tl = Vec2::new(1., 4.) * 8.;
    let br = Vec2::new(2., 5.) * 8.;
    sprite.rect = Some(Rect::from_corners(tl, br));

    if let Ok(mut vis) = blasters.get_single_mut() {
        *vis = Visibility::Hidden;
    }
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
    mut q: Query<(Entity, &mut WeaponEntity, &mut BulletRate), With<Player>>,
    mut events: EventReader<PickupEvent>,
    mut commands: Commands,
) {
    let Ok((player, mut weapon_entity, mut rate)) = q.get_single_mut() else {
        return;
    };

    for event in events.read() {
        match event {
            PickupEvent::Weapon(Weapon::Bullet) => {
                commands.entity(weapon_entity.0).despawn_recursive();

                let emitter = commands
                    .spawn((DualEmitter::<layers::Enemy>::new(), Polarity::North))
                    .id();
                weapon_entity.0 = emitter;
                commands.entity(player).add_child(emitter);
            }
            PickupEvent::Weapon(Weapon::Missile) => {
                commands.entity(weapon_entity.0).despawn_recursive();

                let emitter = commands
                    .spawn((
                        HomingEmitter::<layers::Enemy, Enemy>::new(),
                        Polarity::North,
                    ))
                    .id();
                weapon_entity.0 = emitter;
                commands.entity(player).add_child(emitter);
            }
            PickupEvent::Upgrade(Upgrade::Speed(s)) => rate.0 += *s,
            _ => {}
        }
    }
}
