use crate::Avian;
use crate::enemy::Enemy;
use crate::player::Player;
use avian2d::prelude::*;
use bevy::prelude::*;
use rand::distr::{Distribution, weighted::WeightedIndex};
use std::f32::consts::{PI, TAU};
use std::marker::PhantomData;

pub struct HomingPlugin;

impl Plugin for HomingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (select_target::<Player>, select_target::<Enemy>),
        )
        .add_systems(
            Avian,
            (steer_homing, apply_heading_velocity)
                .chain()
                .before(PhysicsSet::StepSimulation),
        );
    }
}

#[derive(Component)]
#[require(Heading, TurnSpeed)]
pub struct Homing<T>(PhantomData<T>);

impl<T: Component> Homing<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Component)]
pub struct HomingTarget(pub Entity);

fn select_target<T: Component>(
    homing: Query<(Entity, &GlobalTransform), (With<Homing<T>>, Without<HomingTarget>)>,
    targets: Query<(Entity, &GlobalTransform), With<T>>,
    mut commands: Commands,
) {
    let mut rng = rand::rng();

    for (homing, homing_trans) in homing.iter() {
        let homing_trans = homing_trans.compute_transform();
        let mut distances = Vec::new();

        for (target, target_trans) in targets.iter() {
            let target_trans = target_trans.compute_transform();
            distances.push((
                target,
                homing_trans.translation.distance(target_trans.translation),
            ));
        }

        let next_target = if distances.len() == 1 {
            distances[0].0
        } else {
            let max = distances
                .iter()
                .map(|d| d.1)
                .max_by(|a, b| a.total_cmp(b))
                .unwrap_or_default();
            for (_, distance) in distances.iter_mut() {
                *distance = max - *distance;
            }

            let Ok(index) = WeightedIndex::new(distances.iter().map(|d| d.1)) else {
                continue;
            };

            let next_target = index.sample(&mut rng);
            distances[next_target].0
        };

        commands.entity(homing).insert(HomingTarget(next_target));
    }
}

/// Facilitates "steering" behavior, giving enemies a feeling of momentum.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct Heading {
    pub direction: f32,
    pub speed: f32,
}

/// The turning speed for headings.
#[derive(Clone, Copy, PartialEq, Component)]
pub struct TurnSpeed(pub f32);

impl Default for TurnSpeed {
    fn default() -> Self {
        Self(300.0)
    }
}

impl Heading {
    pub fn steer_towards(&mut self, time: &Time<Physics>, turn_speed: f32, from: Vec2, to: Vec2) {
        let desired_direction = (to - from).normalize();
        let desired_angle = desired_direction.y.atan2(desired_direction.x);

        let mut angle_diff = (desired_angle - self.direction) % TAU;
        if angle_diff > PI {
            angle_diff = PI - angle_diff;
        } else if angle_diff < -PI {
            angle_diff = -PI - angle_diff;
        };

        self.direction += angle_diff * turn_speed * time.delta_secs();
        self.direction %= TAU;
    }
}

fn steer_homing(
    mut homing: Query<(
        Entity,
        &GlobalTransform,
        &HomingTarget,
        &mut Heading,
        &TurnSpeed,
    )>,
    targets: Query<&GlobalTransform>,
    time: Res<Time<Physics>>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (entity, transform, target, mut heading, turn_speed) in homing.iter_mut() {
        let Ok(target) = targets.get(target.0) else {
            commands.entity(entity).remove::<HomingTarget>();
            continue;
        };

        let transform = transform.compute_transform();
        let target = target.compute_transform();

        heading.steer_towards(
            &time,
            turn_speed.0 * delta,
            transform.translation.xy(),
            target.translation.xy(),
        );
    }
}

#[derive(Default, Component)]
pub struct HomingRotate;

fn apply_heading_velocity(
    mut homing: Query<(
        &mut Transform,
        &Heading,
        &mut LinearVelocity,
        Option<&HomingRotate>,
    )>,
) {
    for (mut transform, heading, mut velocity, rotate) in homing.iter_mut() {
        velocity.0.x = heading.speed * heading.direction.cos();
        velocity.0.y = heading.speed * heading.direction.sin();

        if rotate.is_some() {
            let new_rotation = Quat::from_rotation_z(heading.direction - PI / 2.0);
            transform.rotation = new_rotation;
        }
    }
}
