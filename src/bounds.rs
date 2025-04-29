use bevy::prelude::*;
use physics::{layers, prelude::*};

pub struct ScreenBoundsPlugin;

impl Plugin for ScreenBoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bounds);
    }
}

#[derive(Component)]
pub struct ScreenBounds;

fn spawn_bounds(mut commands: Commands) {
    let top = crate::HEIGHT / 2.0;
    let bottom = -crate::HEIGHT / 2.0;
    let left = -crate::WIDTH / 2.0;
    let right = crate::WIDTH / 2.0;

    let thickness = 32.0;

    let horizontal_collider = Collider::from_rect(
        Vec2::new(left, thickness / 2.0),
        Vec2::new(crate::WIDTH, thickness),
    );

    // top
    commands.spawn((
        Transform::default().with_translation(Vec3::new(0.0, top + thickness / 2.0, 0.0)),
        ScreenBounds,
        layers::Wall,
        StaticBody,
        horizontal_collider,
        CollisionTrigger(horizontal_collider),
    ));

    // bottom
    commands.spawn((
        Transform::default().with_translation(Vec3::new(0.0, bottom - thickness / 2.0, 0.0)),
        ScreenBounds,
        layers::Wall,
        StaticBody,
        horizontal_collider,
        CollisionTrigger(horizontal_collider),
    ));

    let vertical_collider = Collider::from_rect(Vec2::ZERO, Vec2::new(thickness, crate::HEIGHT));
    // left
    commands.spawn((
        Transform::default().with_translation(Vec3::new(left - thickness, top, 0.0)),
        ScreenBounds,
        layers::Wall,
        StaticBody,
        vertical_collider,
        CollisionTrigger(vertical_collider),
    ));

    // right
    commands.spawn((
        Transform::default().with_translation(Vec3::new(right, top, 0.0)),
        ScreenBounds,
        layers::Wall,
        StaticBody,
        vertical_collider,
        CollisionTrigger(vertical_collider),
    ));
}
