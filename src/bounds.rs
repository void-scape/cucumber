use avian2d::prelude::*;
use bevy::prelude::*;
use crate::Layer;

pub struct ScreenBoundsPlugin;

impl Plugin for ScreenBoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bounds);
    }
}

#[derive(Component)]
#[require(
    RigidBody::Static, 
    CollidingEntities, 
    CollisionLayers::new([Layer::Bounds], [Layer::Player, Layer::Bullet]),
)]
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
    ));

    // bottom
    commands.spawn((
        Transform::default().with_translation(Vec3::new(0., bottom - thickness / 2., 0.)),
        Collider::rectangle(crate::WIDTH, thickness),
        ScreenBounds,
    ));

    // left
    commands.spawn((
        Transform::default().with_translation(Vec3::new(left - thickness / 2., 0., 0.)),
        Collider::rectangle(thickness, crate::HEIGHT),
        ScreenBounds,
    ));

    // right
    commands.spawn((
        Transform::default().with_translation(Vec3::new(right + thickness / 2., 0., 0.)),
        Collider::rectangle(thickness, crate::HEIGHT),
        ScreenBounds,
    ));
}
