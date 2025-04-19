use super::prelude::{Collision, Triggers};
use bevy::prelude::*;
use core::marker::PhantomData;

/// A marker component that can be placed on dynamic bodies
/// to enable collisions between the dynamic body and the
/// target static or dynamic bodies.
#[derive(Debug, Component)]
#[require(Collision<T>)]
pub struct CollidesWith<T: Component>(PhantomData<T>);

impl<T: Component> Default for CollidesWith<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// A marker component that can be placed on any body
/// to enable triggering between the body and the
/// target trigger bodies.
#[derive(Debug, Component)]
#[require(Triggers<T>)]
pub struct TriggersWith<T: Component>(PhantomData<T>);

impl<T: Component> Default for TriggersWith<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Default, Debug, Component)]
pub struct Wall;

#[derive(Default, Debug, Component)]
pub struct Enemy;

#[derive(Default, Debug, Component)]
pub struct Player;

pub trait RegisterPhysicsLayer {
    fn register_trigger_layer<T: Component>(&mut self) -> &mut Self;
    fn register_collision_layer<T: Component>(&mut self) -> &mut Self;
    fn register_grounded_layer<T: Component>(&mut self) -> &mut Self;
    fn register_brushing_layer<T: Component>(&mut self) -> &mut Self;
}

impl RegisterPhysicsLayer for App {
    fn register_trigger_layer<T: Component>(&mut self) -> &mut Self {
        self.add_systems(
            super::Physics,
            super::trigger::handle_triggers::<T>.in_set(super::PhysicsSystems::Collision),
        )
    }

    fn register_collision_layer<T: Component>(&mut self) -> &mut Self {
        self.add_systems(
            super::Physics,
            (
                super::collision::handle_collisions::<T>,
                super::collision::handle_dynamic_body_collsions::<T>,
            )
                .chain()
                .in_set(super::CollisionSystems::Resolution),
        )
    }

    fn register_grounded_layer<T: Component>(&mut self) -> &mut Self {
        self.add_systems(
            super::Physics,
            super::collision::update_grounded::<T>
                .chain()
                .in_set(super::CollisionSystems::Grounding),
        )
    }

    fn register_brushing_layer<T: Component>(&mut self) -> &mut Self {
        self.add_systems(
            super::Physics,
            super::collision::update_brushing::<T>
                .chain()
                .in_set(super::CollisionSystems::Brushing),
        )
    }
}
