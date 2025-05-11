use std::time::Duration;

use crate::auto_collider::ImageCollider;
use crate::bounds::WallDespawn;
use crate::player::Player;
use crate::{Layer, assets};
use avian2d::prelude::*;
use bevy::color::palettes::css::YELLOW;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use rand::Rng;
use rand::seq::IteratorRandom;

const PICKUP_SPEED: f32 = 16.;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickupEvent>()
            //.add_systems(Startup, debug)
            .add_systems(Update, (pickup_triggered, update_scrolling_pickup));
    }
}

fn debug(mut commands: Commands) {
    commands.spawn((Upgrade::Speed(0.5), Transform::from_xyz(0., 30., 0.)));
    commands.spawn((Upgrade::Juice(0.5), Transform::from_xyz(0., -30., 0.)));

    commands.spawn((Weapon::Bullet, Transform::from_xyz(30., 0., 0.)));
    commands.spawn((Weapon::Laser, Transform::from_xyz(-30., 0., 0.)));
    commands.spawn((Weapon::Missile, Transform::from_xyz(0., 50., 0.)));
}

pub fn velocity() -> LinearVelocity {
    LinearVelocity(Vec2::NEG_Y * PICKUP_SPEED)
}

#[derive(Debug, Event)]
pub enum PickupEvent {
    Weapon(Weapon),
    Upgrade(Upgrade),
    Material,
}

impl From<Pickup> for PickupEvent {
    fn from(value: Pickup) -> Self {
        match value {
            Pickup::Upgrade(upgrade) => Self::Upgrade(upgrade),
            Pickup::Weapon(weapon) => Self::Weapon(weapon),
        }
    }
}

impl From<&Pickup> for PickupEvent {
    fn from(value: &Pickup) -> Self {
        match *value {
            Pickup::Upgrade(upgrade) => Self::Upgrade(upgrade),
            Pickup::Weapon(weapon) => Self::Weapon(weapon),
        }
    }
}

#[derive(Default, Component)]
#[require(
    Transform,
    RigidBody::Kinematic,
    Sensor,
    WallDespawn,
    CollisionLayers::new(Layer::Collectable, [Layer::Bounds, Layer::Player, Layer::Miners]),
)]
pub struct Collectable;

fn pickup_triggered(
    mut commands: Commands,
    mut writer: EventWriter<PickupEvent>,
    player: Single<&CollidingEntities, With<Player>>,
    collectables: Query<&Collectable>,
    upgrades: Query<&Upgrade>,
    weapons: Query<&Weapon>,
) {
    for entity in player
        .iter()
        .copied()
        .filter(|entity| collectables.get(*entity).is_ok())
    {
        if let Ok(upgrade) = upgrades.get(entity) {
            writer.write(PickupEvent::Upgrade(*upgrade));
        } else if let Ok(weapon) = weapons.get(entity) {
            writer.write(PickupEvent::Weapon(*weapon));
        } else {
            continue;
        }

        commands.entity(entity).despawn();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
#[component(on_add = Self::sprite_hook)]
pub enum Upgrade {
    Speed(f32),
    Juice(f32),
}

impl Upgrade {
    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Speed(_) => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(2, 1)),
            Self::Juice(_) => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(3, 1)),
        }
    }
}

impl SpriteHook for Upgrade {
    fn sprite(&self, server: &AssetServer) -> Sprite {
        self.sprite(server)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
#[component(on_add = Self::sprite_hook)]
pub enum Weapon {
    Bullet,
    Laser,
    Missile,
}

impl Weapon {
    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Bullet => {
                assets::sprite_rect8(server, assets::PROJECTILES_COLORED_PATH, UVec2::new(0, 1))
            }
            Self::Laser => {
                assets::sprite_rect8(server, assets::PROJECTILES_COLORED_PATH, UVec2::new(1, 8))
            }
            Self::Missile => {
                assets::sprite_rect8(server, assets::PROJECTILES_COLORED_PATH, UVec2::new(5, 2))
            }
        }
    }
}

impl SpriteHook for Weapon {
    fn sprite(&self, server: &AssetServer) -> Sprite {
        self.sprite(server)
    }
}

pub trait SpriteHook
where
    Self: Sized + Component,
{
    fn sprite(&self, server: &AssetServer) -> Sprite;

    fn sprite_hook(mut world: DeferredWorld, ctx: HookContext) {
        let t = world.entity(ctx.entity).get::<Self>().unwrap();
        let sprite = t.sprite(world.get_resource::<AssetServer>().unwrap());
        world.commands().entity(ctx.entity).insert(sprite);
    }
}

pub fn random_pickups(num: usize) -> Vec<Pickup> {
    (0..num).map(|_| Pickup::random()).collect()
}

pub fn unique_pickups(num: usize) -> Vec<Pickup> {
    let mut pickups = Vec::with_capacity(num);
    while pickups.len() != 3 {
        let pickup = Pickup::random();
        if !pickups.contains(&pickup) {
            pickups.push(pickup);
        }
    }
    pickups
}

pub fn spawn_random_pickup(commands: &mut EntityCommands, bundle: impl Bundle) {
    match Pickup::random() {
        Pickup::Upgrade(upgrade) => commands.insert((upgrade, bundle)),
        Pickup::Weapon(weapon) => commands.insert((weapon, bundle)),
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum Pickup {
    Upgrade(Upgrade),
    Weapon(Weapon),
}

impl Pickup {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        [
            Self::Upgrade(Upgrade::Speed(0.2)),
            Self::Upgrade(Upgrade::Juice(0.2)),
            Self::Upgrade(Upgrade::Speed(0.2)),
            Self::Upgrade(Upgrade::Juice(0.2)),
            Self::Weapon(Weapon::Bullet),
            Self::Weapon(Weapon::Missile),
            Self::Weapon(Weapon::Laser),
        ]
        .into_iter()
        .choose(&mut rng)
        .unwrap()
    }
}

#[derive(Component)]
#[require(DebugCircle::color(2., YELLOW), ImageCollider, Collectable)]
pub struct Material;

#[derive(Component)]
#[require(Collectable, Collider::rectangle(8., 8.), LinearVelocity(Vec2::NEG_Y * 20.))]
pub struct ScrollingPickup {
    index: usize,
    timer: Timer,
}

impl ScrollingPickup {
    pub fn new() -> Self {
        Self {
            index: rand::rng().random_range(0..5),
            timer: Timer::from_seconds(2., TimerMode::Repeating),
        }
    }
}

fn update_scrolling_pickup(
    mut commands: Commands,
    time: Res<Time>,
    mut pickup: Query<(Entity, &mut ScrollingPickup)>,
) {
    for (entity, mut scroll_pickup) in pickup.iter_mut() {
        scroll_pickup.timer.tick(time.delta());
        if scroll_pickup.is_added() {
            let dur = scroll_pickup.timer.duration();
            scroll_pickup.timer.set_elapsed(dur);
        }

        if scroll_pickup.timer.just_finished() {
            scroll_pickup.index += 1;
            if scroll_pickup.index > 4 {
                scroll_pickup.index = 0;
            }

            let pickup = [
                Pickup::Upgrade(Upgrade::Speed(0.2)),
                Pickup::Upgrade(Upgrade::Juice(0.2)),
                Pickup::Weapon(Weapon::Bullet),
                Pickup::Weapon(Weapon::Missile),
                Pickup::Weapon(Weapon::Laser),
            ]
            .into_iter()
            .nth(scroll_pickup.index)
            .unwrap();
            commands.entity(entity).remove::<(Weapon, Upgrade)>();

            match pickup {
                Pickup::Weapon(weapon) => {
                    commands.entity(entity).insert(weapon);
                }
                Pickup::Upgrade(upgrade) => {
                    commands.entity(entity).insert(upgrade);
                }
            }
        }
    }
}
