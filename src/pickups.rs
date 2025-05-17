use crate::auto_collider::ImageCollider;
use crate::bounds::WallDespawn;
use crate::player::Player;
use crate::sprites::CellSprite;
use crate::{DespawnRestart, Layer, assets, sprites};
use avian2d::prelude::*;
use bevy::color::palettes::css::{LIGHT_BLUE, YELLOW};
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_optix::debug::{DebugCircle, DebugRect};
use rand::Rng;

const PICKUP_SPEED: f32 = 16.;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickupEvent>()
            .add_systems(Update, (pickup_triggered, update_scrolling_pickup));
    }
}

#[derive(Debug, Event)]
pub enum PickupEvent {
    Weapon(Weapon),
    Upgrade(Upgrade),
    Material(Material),
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
    DespawnRestart,
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
//#[component(on_add = Self::sprite_hook)]
pub enum Upgrade {
    Speed(f32),
    Juice(f32),
}

//impl Upgrade {
//    pub fn sprite(&self, server: &AssetServer) -> Sprite {
//        match self {
//            Self::Speed(_) => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(2, 1)),
//            Self::Juice(_) => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(3, 1)),
//        }
//    }
//}
//
//impl SpriteHook for Upgrade {
//    fn sprite(&self, _server: &AssetServer) -> Sprite {
//        panic!();
//    }
//}

#[derive(Default, Debug, Clone, Copy, PartialEq, Component)]
#[component(on_add = Self::sprite_hook)]
pub enum Weapon {
    #[default]
    Bullet,
    //Laser,
    Missile,
}

impl Weapon {
    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Bullet => sprites::sprite_rect(
                server,
                assets::PROJECTILES_COLORED_PATH,
                sprites::CellSize::Eight,
                UVec2::new(0, 1),
            ),
            Self::Missile => sprites::sprite_rect(
                server,
                assets::PROJECTILES_COLORED_PATH,
                sprites::CellSize::Eight,
                UVec2::new(5, 2),
            ),
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

//pub fn random_pickups(num: usize) -> Vec<Pickup> {
//    (0..num).map(|_| Pickup::random()).collect()
//}
//
//pub fn unique_pickups(num: usize) -> Vec<Pickup> {
//    let mut pickups = Vec::with_capacity(num);
//    while pickups.len() != 3 {
//        let pickup = Pickup::random();
//        if !pickups.contains(&pickup) {
//            pickups.push(pickup);
//        }
//    }
//    pickups
//}
//
//pub fn spawn_random_pickup(commands: &mut EntityCommands, bundle: impl Bundle) {
//    match Pickup::random() {
//        Pickup::Upgrade(upgrade) => commands.insert((upgrade, bundle)),
//        Pickup::Weapon(weapon) => commands.insert((weapon, bundle)),
//    };
//}

#[derive(Debug, Clone, Copy, PartialEq, Component)]
#[require(ImageCollider, Collectable, LinearVelocity(Vec2::NEG_Y * PICKUP_SPEED))]
pub enum Pickup {
    Upgrade(Upgrade),
    Weapon(Weapon),
}

//impl Pickup {
//    pub fn random() -> Self {
//        let mut rng = rand::rng();
//        [
//            Self::Upgrade(Upgrade::Speed(0.2)),
//            Self::Upgrade(Upgrade::Juice(0.2)),
//            Self::Upgrade(Upgrade::Speed(0.2)),
//            Self::Upgrade(Upgrade::Juice(0.2)),
//            Self::Weapon(Weapon::Bullet),
//            Self::Weapon(Weapon::Missile),
//            Self::Weapon(Weapon::Laser),
//        ]
//        .into_iter()
//        .choose(&mut rng)
//        .unwrap()
//    }
//}

#[derive(Debug, Clone, Copy, Component)]
#[require(ImageCollider, Collectable)]
#[component(on_add = Self::insert_visual)]
pub enum Material {
    Parts,
    Shield,
}

impl Material {
    fn insert_visual(mut world: DeferredWorld, ctx: HookContext) {
        let mat = *world.get::<Material>(ctx.entity).unwrap();
        world.commands().entity(ctx.entity).insert(match mat {
            Material::Parts => DebugCircle::color(2., YELLOW),
            Material::Shield => DebugCircle::color(2., LIGHT_BLUE),
        });
    }
}

#[derive(Component)]
#[require(Collectable, Collider::rectangle(8., 8.), LinearVelocity(Vec2::NEG_Y * 20.), DespawnRestart)]
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

        const CHOICES: [Pickup; 4] = [
            Pickup::Upgrade(Upgrade::Speed(0.2)),
            Pickup::Upgrade(Upgrade::Juice(0.2)),
            Pickup::Weapon(Weapon::Bullet),
            Pickup::Weapon(Weapon::Missile),
            //Pickup::Weapon(Weapon::Laser),
        ];

        //if scroll_pickup.timer.just_finished() {
        //    scroll_pickup.index += 1;
        //    if scroll_pickup.index >= CHOICES.len() {
        //        scroll_pickup.index = 0;
        //    }
        //
        //    let pickup = CHOICES.into_iter().nth(scroll_pickup.index).unwrap();
        //    commands.entity(entity).remove::<(Weapon, Upgrade)>();
        //
        //    match pickup {
        //        Pickup::Weapon(weapon) => {
        //            commands.entity(entity).insert(weapon);
        //        }
        //        Pickup::Upgrade(upgrade) => {
        //            commands.entity(entity).insert(upgrade);
        //        }
        //    }
        //}
    }
}

#[derive(Debug, Clone, Copy, Component)]
#[require(
    Collectable,
    Collider::circle(8.),
    CellSprite::new16("ships.png", UVec2::new(3, 0)),
    LinearVelocity(Vec2::NEG_Y * PICKUP_SPEED),
    AngularVelocity(0.1),
)]
pub struct PowerUp;

#[derive(Debug, Clone, Copy, Component)]
#[require(
    Collectable,
    Collider::circle(8.),
    CellSprite::new16("ships.png", UVec2::new(2, 0)),
    LinearVelocity(Vec2::NEG_Y * PICKUP_SPEED),
    AngularVelocity(0.1),
)]
pub struct Bomb;
