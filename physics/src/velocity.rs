use super::TimeScale;
use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, Component)]
#[require(Acceleration)]
pub struct Velocity(pub Vec2);

/// Describes the absolute max active velocity in both the x and y axis.
#[derive(Debug, Clone, Copy, Component)]
pub struct MaxVelocity(pub Vec2);

#[derive(Debug, Default, Clone, Component)]
#[require(Mass)]
pub struct Acceleration {
    forces: Vec<Vec2>,
}

impl Acceleration {
    pub fn apply_force(&mut self, force: Vec2) {
        self.forces.push(force);
    }

    pub fn apply(
        &self,
        weight: &Mass,
        velocity: &mut Velocity,
        max: Option<&MaxVelocity>,
        scale: &TimeScale,
    ) {
        self.forces
            .iter()
            .map(|f| f / weight.0)
            .for_each(|a| velocity.0 += a * scale.0);

        if let Some(max) = max {
            let maxabs = max.0.abs();
            let velabs = velocity.0.abs();

            if maxabs.x < velabs.x {
                velocity.0.x = velocity.0.x.signum() * maxabs.x;
            }

            if maxabs.y < velabs.y {
                velocity.0.y = velocity.0.y.signum() * maxabs.y;
            }
        }
    }
}

/// Entity mass.
#[derive(Debug, Clone, Copy, Component)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.)
    }
}

pub fn apply_velocity(
    mut query: Query<(
        &mut Transform,
        &mut Velocity,
        &mut Acceleration,
        &Mass,
        Option<&MaxVelocity>,
    )>,
    time: Res<Time>,
    scale: Single<&TimeScale>,
) {
    for (mut transform, mut velocity, mut acceleration, weight, max) in query.iter_mut() {
        // It doesn't seem like scale should be applied to acceleration, but it does.
        acceleration.apply(weight, &mut velocity, max, &scale);
        transform.translation += (velocity.0 * time.delta_secs() * scale.0).extend(0.);
        acceleration.forces.clear();
    }
}
