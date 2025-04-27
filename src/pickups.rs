use crate::assets;
use crate::player::Player;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use physics::layers::RegisterPhysicsLayer;
use physics::prelude::{Collider, Velocity};
use physics::trigger::{CollisionTrigger, Triggers};
use rand::seq::IteratorRandom;

const PICKUP_SPEED: f32 = 16.;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickupEvent>()
            .register_trigger_layer::<PickupLayer>()
            //.add_systems(Startup, debug)
            .add_systems(Update, pickup_triggered);
    }
}

fn debug(mut commands: Commands) {
    commands.spawn((Upgrade::Speed(0.5), Transform::from_xyz(0., 30., 0.)));
    commands.spawn((Upgrade::Juice(0.5), Transform::from_xyz(0., -30., 0.)));

    commands.spawn((Weapon::Bullet, Transform::from_xyz(30., 0., 0.)));
    commands.spawn((Weapon::Laser, Transform::from_xyz(-30., 0., 0.)));
    commands.spawn((Weapon::Missile, Transform::from_xyz(0., 50., 0.)));
}

pub fn velocity() -> Velocity {
    Velocity(Vec2::NEG_Y * PICKUP_SPEED)
}

#[derive(Event)]
pub enum PickupEvent {
    Weapon(Weapon),
    Upgrade(Upgrade),
}

fn pickup_triggered(
    mut commands: Commands,
    mut writer: EventWriter<PickupEvent>,
    player: Single<&Triggers<PickupLayer>, With<Player>>,
    upgrades: Query<(Entity, &Upgrade)>,
    weapons: Query<(Entity, &Weapon)>,
) {
    for entity in player.entities().iter() {
        if let Some((_, upgrade)) = upgrades.iter().find(|(upgrade, _)| entity == upgrade) {
            commands.entity(*entity).despawn();
            writer.write(PickupEvent::Upgrade(*upgrade));
        }

        if let Some((_, weapon)) = weapons.iter().find(|(weapon, _)| entity == weapon) {
            commands.entity(*entity).despawn();
            writer.write(PickupEvent::Weapon(*weapon));
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Transform, PickupLayer, CollisionTrigger = upgrade_trigger())]
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

#[derive(Clone, Copy, Component)]
#[require(Transform, PickupLayer, CollisionTrigger = weapon_trigger())]
#[component(on_add = Self::sprite_hook)]
pub enum Weapon {
    Bullet,
    Laser,
    Missile,
}

impl Weapon {
    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Bullet => assets::sprite_rect16(server, assets::WEAPONS_PATH, UVec2::new(0, 0)),
            Self::Laser => assets::sprite_rect16(server, assets::WEAPONS_PATH, UVec2::new(2, 0)),
            Self::Missile => assets::sprite_rect16(server, assets::WEAPONS_PATH, UVec2::new(1, 0)),
        }
    }
}

impl SpriteHook for Weapon {
    fn sprite(&self, server: &AssetServer) -> Sprite {
        self.sprite(server)
    }
}

#[derive(Default, Component)]
pub struct PickupLayer;

fn upgrade_trigger() -> CollisionTrigger {
    let tl = -(Vec2::X + Vec2::NEG_Y) * 4.;
    let size = Vec2::ONE * 8.;
    CollisionTrigger(Collider::from_rect(tl, size))
}

fn weapon_trigger() -> CollisionTrigger {
    let tl = -(Vec2::X + Vec2::NEG_Y) * 8.;
    let size = Vec2::ONE * 16.;
    CollisionTrigger(Collider::from_rect(tl, size))
}

trait SpriteHook
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

pub fn spawn_random_pickup(commands: &mut EntityCommands, bundle: impl Bundle) {
    match Pickup::random() {
        Pickup::Upgrade(upgrade) => commands.insert((upgrade, bundle)),
        Pickup::Weapon(weapon) => commands.insert((weapon, bundle)),
    };
}

enum Pickup {
    Upgrade(Upgrade),
    Weapon(Weapon),
}

impl Pickup {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        [
            Self::Upgrade(Upgrade::Speed(0.5)),
            Self::Upgrade(Upgrade::Juice(0.5)),
            Self::Upgrade(Upgrade::Speed(0.5)),
            Self::Upgrade(Upgrade::Juice(0.5)),
            Self::Weapon(Weapon::Bullet),
            Self::Weapon(Weapon::Missile),
            Self::Weapon(Weapon::Laser),
        ]
        .into_iter()
        .choose(&mut rng)
        .unwrap()
    }
}
