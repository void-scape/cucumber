use bevy::prelude::*;

#[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum MovementSystems {
    Velocity,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PostUpdate, MovementSystems::Velocity)
            .add_systems(PostUpdate, apply_velocity.in_set(MovementSystems::Velocity));
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Velocity(pub Vec2);

fn apply_velocity(mut q: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    let delta = time.delta().as_secs_f32();
    for (Velocity(velocity), mut transform) in q.iter_mut() {
        transform.translation.x += velocity.x * delta;
        transform.translation.y += velocity.y * delta;
    }
}
