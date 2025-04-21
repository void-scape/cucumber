use crate::assets;
use crate::player::Player;
use bevy::ecs::component::ComponentId;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use physics::layers::RegisterPhysicsLayer;
use physics::prelude::Collider;
use physics::trigger::{CollisionTrigger, Triggers};

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

#[derive(Event)]
pub enum PickupEvent {
    Weapon(Weapon),
    Upgrade(Upgrade),
}

fn pickup_triggered(
    mut commands: Commands,
    mut writer: EventWriter<PickupEvent>,
    player: Query<&Triggers<PickupLayer>, With<Player>>,
    upgrades: Query<(Entity, &Upgrade)>,
    weapons: Query<(Entity, &Weapon)>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    for entity in player.entities().iter() {
        if let Some((_, upgrade)) = upgrades.iter().find(|(upgrade, _)| entity == upgrade) {
            commands.entity(*entity).despawn_recursive();
            writer.send(PickupEvent::Upgrade(*upgrade));
        }

        if let Some((_, weapon)) = weapons.iter().find(|(weapon, _)| entity == weapon) {
            commands.entity(*entity).despawn_recursive();
            writer.send(PickupEvent::Weapon(*weapon));
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Transform, PickupLayer, CollisionTrigger(upgrade_trigger))]
#[component(on_add = Self::on_add)]
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
#[require(Transform, PickupLayer, CollisionTrigger(weapon_trigger))]
#[component(on_add = Self::on_add)]
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

    fn on_add(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let t = world.entity(entity).get::<Self>().unwrap();
        let sprite = t.sprite(world.get_resource::<AssetServer>().unwrap());
        world.commands().entity(entity).insert(sprite);
    }
}
