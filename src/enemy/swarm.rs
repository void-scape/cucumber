use crate::WIDTH;
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

const SWARM_SPEED: f32 = 60.;
const SWARM_THRESHOLD: f32 = WIDTH * 1.25;

#[derive(Debug, Component)]
pub enum SwarmMovement {
    Left,
    Right,
}

pub(super) fn swarm_movement(
    mut q: Query<(&mut LinearVelocity, &GlobalTransform, &SwarmMovement)>,
) {
    for (mut velocity, transform, movement) in &mut q {
        let translation = transform.compute_transform().translation;

        match movement {
            SwarmMovement::Left => {
                if translation.x > SWARM_THRESHOLD && velocity.x > 0. {
                    velocity.x = -SWARM_SPEED;
                } else if translation.x < -SWARM_THRESHOLD && velocity.x < 0. {
                    velocity.x = SWARM_SPEED;
                } else if velocity.x == 0. {
                    velocity.x = SWARM_SPEED;
                }
            }
            SwarmMovement::Right => {
                if translation.x < SWARM_THRESHOLD && velocity.x < 0. {
                    velocity.x = -SWARM_SPEED;
                } else if translation.x > -SWARM_THRESHOLD && velocity.x > 0. {
                    velocity.x = SWARM_SPEED;
                } else if velocity.x == 0. {
                    velocity.x = SWARM_SPEED;
                }
            }
        }

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
