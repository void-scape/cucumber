use crate::prelude::Velocity;

use super::prelude::Acceleration;
use bevy::prelude::*;

/// Global definition of the gravity force.
#[derive(Debug, Clone, Copy, Resource)]
pub struct Gravity(pub Vec2);

/// An entity who is not falling.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Grounded;

/// An entity that's brushing a left or right wall.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct BrushingLeft;

/// An entity that's brushing a left or right wall.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct BrushingRight;

/// An entity who experiences [`Gravity`].
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Gravitational;

pub fn apply_gravity(
    gravity: Res<Gravity>,
    mut object_query: Query<
        (&mut Acceleration, &Velocity),
        (With<Gravitational>, Without<Grounded>),
    >,
) {
    let max_gravity = -300.;
    for (mut acceleration, velocity) in object_query.iter_mut() {
        if velocity.0.y > max_gravity {
            acceleration.apply_force(gravity.0);
        }
    }
}
