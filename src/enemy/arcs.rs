use super::Enemy;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::arcs::ArcsEmitter;
use crate::enemy::FaceVelocity;
use crate::tween::DespawnTweenFinish;
use crate::{
    Layer,
    auto_collider::ImageCollider,
    bullet::emitter::{BulletModifiers, Rate},
    effects::Explosion,
    health::Health,
};
use avian2d::prelude::*;
use bevy::color::palettes::css::GRAY;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    ArcsEmitter,
    Health::full(50.),
    DebugCircle::color(12., GRAY),
    CollisionLayers::new([Layer::Enemy], [Layer::Bullet, Layer::Player]),
    BulletModifiers {
        rate: Rate::Factor(0.2),
        ..Default::default()
    },
    Trauma(0.04),
    Explosion::Small,
    DespawnTweenFinish,
    FaceVelocity,
)]
pub struct Arcs;

// This one pivoted haflfway through
pub fn persistent() -> Formation {
    Formation::new(move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            let platoon = root.target_entity();

            root.spawn((
                Arcs,
                Platoon(platoon),
                EmitterDelay::new(1.),
                Transform::from_xyz(0., 0., 0.),
            ));
        });
    })
}
