use bevy::app::FixedMainScheduleOrder;
use bevy::render::camera::PhysicalCameraParameters;
use bevy::sprite::Wireframe2dPlugin;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use bevy_tween::prelude::Interpolator;
use bevy_tween::{component_tween_system, BevyTweenRegisterSystems};
use layers::RegisterPhysicsLayer;

pub mod collision;
pub mod debug;
pub mod gravity;
pub mod layers;
pub mod spatial;
pub mod trigger;
pub mod velocity;

#[allow(unused)]
pub mod prelude {
    pub use super::collision::*;
    pub use super::gravity::*;
    pub use super::layers;
    pub use super::trigger::*;
    pub use super::velocity::*;
}

#[derive(Debug, Component)]
pub struct TimeScale(pub f32);

#[derive(Debug, Component)]
pub struct TimeScaleRate {
    start: f32,
    end: f32,
}

impl TimeScaleRate {
    pub fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }
}

impl Interpolator for TimeScaleRate {
    type Item = TimeScale;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.0 = self.start.lerp(self.end, value);
    }
}

pub fn time_scale(start: f32, end: f32) -> TimeScaleRate {
    TimeScaleRate::new(start, end)
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct Physics;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub enum CollisionSystems {
    Resolution,
    Grounding,
    Brushing,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub enum PhysicsSystems {
    Velocity,
    Collision,
}

#[derive(Debug)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(Physics);
        app.world_mut()
            .resource_mut::<FixedMainScheduleOrder>()
            .insert_after(FixedUpdate, Physics);

        app.world_mut().spawn(TimeScale(1.));

        app.register_collision_layer::<layers::Player>()
            .register_collision_layer::<layers::Enemy>()
            .register_collision_layer::<layers::Wall>()
            .register_grounded_layer::<layers::Wall>()
            .register_brushing_layer::<layers::Wall>();

        app.add_tween_systems(component_tween_system::<TimeScaleRate>())
            .add_plugins(Wireframe2dPlugin)
            .add_event::<trigger::TriggerEvent>()
            .add_event::<trigger::TriggerEnter>()
            .add_event::<trigger::TriggerExit>()
            .init_resource::<collision::TilesetSize>()
            .insert_resource(debug::ShowCollision(false))
            .add_systems(Update, collision::build_tile_set_colliders)
            .add_systems(
                Physics,
                (
                    collision::clear_resolution
                        .in_set(PhysicsSystems::Collision)
                        .before(CollisionSystems::Resolution),
                    (gravity::apply_gravity, velocity::apply_velocity)
                        .chain()
                        .in_set(PhysicsSystems::Velocity),
                    (
                        bevy::transform::systems::sync_simple_transforms,
                        bevy::transform::systems::propagate_transforms,
                        spatial::store_static_body_in_spatial_map,
                    )
                        .chain()
                        .before(PhysicsSystems::Collision)
                        .after(PhysicsSystems::Velocity),
                    (
                        trigger::emit_trigger_states,
                        debug::debug_display_collider_wireframe,
                        debug::update_show_collision,
                        (
                            debug::debug_show_collision_color,
                            debug::debug_show_trigger_color,
                        )
                            .chain(),
                    )
                        .in_set(PhysicsSystems::Collision),
                ),
            )
            .configure_sets(
                Physics,
                (
                    PhysicsSystems::Velocity.before(PhysicsSystems::Collision),
                    CollisionSystems::Resolution
                        .before(CollisionSystems::Grounding)
                        .before(CollisionSystems::Brushing),
                    CollisionSystems::Resolution.in_set(PhysicsSystems::Collision),
                    CollisionSystems::Grounding.in_set(PhysicsSystems::Collision),
                    CollisionSystems::Brushing.in_set(PhysicsSystems::Collision),
                ),
            );
    }
}
