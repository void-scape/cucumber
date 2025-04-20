use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use physics::prelude::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct BulletTimer {
    pub timer: Timer,
}

#[derive(Clone, Copy, Component, Default)]
#[require(Velocity, DynamicBody)]
pub struct Bullet;

#[derive(Clone, Copy, Component)]
#[require(Bullet)]
#[component(on_add = Self::on_add)]
pub enum BulletType {
    Basic,
}

impl BulletType {
    fn on_add(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let ty = *world.get::<BulletType>(entity).unwrap();

        match ty {
            BulletType::Basic => {
                world.commands().entity(entity).insert(BasicBullet);
            }
        }
    }
}

#[derive(Clone, Copy, Component)]
#[require(Collider(basic_collider))]
#[component(on_add = on_add_basic)]
pub struct BasicBullet;

fn basic_collider() -> Collider {
    Collider::from_rect(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))
}

fn on_add_basic(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let server = world.get_resource::<AssetServer>().unwrap();

    let bullet = server.load("sprites/bullet.png");

    world
        .commands()
        .entity(entity)
        .insert(Sprite::from_image(bullet));
}
