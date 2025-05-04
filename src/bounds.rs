use crate::Layer;
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct ScreenBoundsPlugin;

impl Plugin for ScreenBoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bounds)
            .add_systems(Update, kill_on_wall);
    }
}

#[derive(Default, Component)]
pub struct WallDespawn;

fn kill_on_wall(
    mut commands: Commands,
    bounds: Query<&CollidingEntities, With<ScreenBounds>>,
    entities: Query<Entity, With<WallDespawn>>,
) {
    for colliding_entities in bounds.iter() {
        for entity in colliding_entities
            .iter()
            .copied()
            .flat_map(|entity| entities.get(entity))
        {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
#[require(RigidBody::Static, CollidingEntities)]
pub struct ScreenBounds;

fn spawn_bounds(mut commands: Commands) {
    let top = crate::HEIGHT / 2.;
    let bottom = -crate::HEIGHT / 2.;
    let left = -crate::WIDTH / 2.;
    let right = crate::WIDTH / 2.;
    let thickness = 32.;

    // top
    commands.spawn((
        Transform::default().with_translation(Vec3::new(0., top + thickness / 2., 0.)),
        Collider::rectangle(crate::WIDTH, thickness),
        ScreenBounds,
        CollisionLayers::new(
            Layer::Bounds,
            [Layer::Player, Layer::Bullet, Layer::Collectable],
        ),
    ));

    // bottom
    commands.spawn((
        Transform::default().with_translation(Vec3::new(0., bottom - thickness / 2., 0.)),
        Collider::rectangle(crate::WIDTH, thickness),
        ScreenBounds,
        CollisionLayers::new(
            Layer::Bounds,
            [Layer::Player, Layer::Bullet, Layer::Collectable],
        ),
    ));

    // left
    commands.spawn((
        Transform::default().with_translation(Vec3::new(left - thickness / 2., 0., 0.)),
        Collider::rectangle(thickness, crate::HEIGHT),
        ScreenBounds,
        CollisionLayers::new(
            Layer::Bounds,
            [Layer::Player, Layer::Bullet, Layer::Collectable],
        ),
    ));

    // right
    commands.spawn((
        Transform::default().with_translation(Vec3::new(right + thickness / 2., 0., 0.)),
        Collider::rectangle(thickness, crate::HEIGHT),
        ScreenBounds,
        CollisionLayers::new(
            Layer::Bounds,
            [Layer::Player, Layer::Bullet, Layer::Collectable],
        ),
    ));
}
