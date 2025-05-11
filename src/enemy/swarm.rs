use crate::WIDTH;
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

const SWARM_SPEED: f32 = 60.;
const SWARM_THRESHOLD: f32 = WIDTH * 2.;

#[derive(Debug, Component)]
pub struct SwarmMovement;

pub(super) fn swarm_movement(
    mut q: Query<(&mut LinearVelocity, &GlobalTransform), With<SwarmMovement>>,
) {
    for (mut velocity, transform) in &mut q {
        let translation = transform.compute_transform().translation;

        if translation.x > SWARM_THRESHOLD && velocity.x > 0. {
            velocity.x = -SWARM_SPEED;
        } else if translation.x < -SWARM_THRESHOLD && velocity.x < 0. {
            velocity.x = SWARM_SPEED;
        } else if velocity.x == 0. {
            velocity.x = SWARM_SPEED;
        }

        let direction = velocity.x.signum();

        if (-WIDTH * 0.5..WIDTH * 0.5).contains(&translation.x) {
            let proportion = (translation.x + WIDTH * 0.5) / WIDTH;

            let proportion = if velocity.x > 0. {
                proportion
            } else {
                1. - proportion
            };

            let sin = ((proportion * core::f32::consts::PI).cos() - 0.5) * -10.;
            velocity.y = sin;
        } else {
            velocity.y = 0.;
        }
    }
}
