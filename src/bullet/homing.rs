use std::{f32::consts::PI, marker::PhantomData};

use bevy::prelude::*;
use physics::{Physics, PhysicsSystems, layers, prelude::Velocity};
use rand::distr::{Distribution, weighted::WeightedIndex};

pub struct HomingPlugin;

impl Plugin for HomingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                select_target::<layers::Player>,
                select_target::<layers::Enemy>,
            ),
        )
        .add_systems(Physics, steer_homing.before(PhysicsSystems::Velocity));
    }
}

#[derive(Component)]
#[require(Heading)]
pub struct Homing<T>(PhantomData<T>);

impl<T: Component> Homing<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Component)]
struct HomingTarget(Entity);

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

        commands
            .entity(homing)
            .insert(HomingTarget(distances[next_target].0));
    }
}

/// Facilitates "steering" behavior, giving enemies a feeling of momentum.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct Heading {
    pub direction: f32,
    pub speed: f32,
}

impl Heading {
    pub fn new(speed: f32) -> Self {
        Self {
            direction: Default::default(),
            speed,
        }
    }
}

/// The turning speed for headings.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct TurnSpeed(pub f32);

impl Heading {
    pub fn steer_towards(&mut self, turn_speed: f32, from: &Vec3, to: &Vec3) {
        use std::f32::consts::{PI, TAU};

        // Calculate the desired direction
        let desired_direction = (*to - *from).normalize();

        // Convert the desired direction to an angle
        let desired_angle = desired_direction.y.atan2(desired_direction.x);

        // Calculate the smallest angle difference
        let mut angle_diff = (desired_angle - self.direction) % TAU;
        if angle_diff > PI {
            angle_diff = PI - angle_diff;
        } else if angle_diff < -PI {
            angle_diff = -PI - angle_diff;
        };

        // Gradually adjust the current direction
        self.direction += angle_diff * turn_speed;

        // Normalize
        self.direction %= TAU;
    }
}

fn steer_homing(
    mut homing: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &HomingTarget,
        &mut Heading,
        &mut Velocity,
    )>,
    targets: Query<&GlobalTransform>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();

    for (entity, transform, mut local_transform, target, mut heading, mut velocity) in
        homing.iter_mut()
    {
        let Ok(target) = targets.get(target.0) else {
            commands.entity(entity).remove::<HomingTarget>();
            continue;
        };

        let transform = transform.compute_transform();
        let target = target.compute_transform();

        heading.steer_towards(2.0 * delta, &transform.translation, &target.translation);

        velocity.0.x = heading.speed * heading.direction.cos();
        velocity.0.y = heading.speed * heading.direction.sin();
        // local_transform.rotation.rot = heading.direction;

        let new_rotation = Quat::from_rotation_z(heading.direction - PI / 2.0);
        local_transform.rotation = new_rotation;
    }
}
