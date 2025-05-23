use super::Enemy;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::ShotLimit;
use crate::enemy::FaceVelocity;
use crate::enemy::swarm::SwarmEmitter;
use crate::tween::DespawnTweenFinish;
use crate::{
    auto_collider::ImageCollider, effects::Explosion, health::Health, sprites::CellSprite,
};
use bevy::prelude::*;
use bevy_tween::combinator::sequence;
use bevy_tween::combinator::tween;
use bevy_tween::prelude::AnimationBuilderExt;
use bevy_tween::prelude::EaseKind;
use bevy_tween::tween::IntoTarget;
use std::time::Duration;

#[derive(Default, Component)]
#[require(
    Enemy,
    ImageCollider,
    Health::full(1.),
    CellSprite::new8("shooters/SpaceShooterAssetPack_Ships.png", UVec2::new(4, 1)),
    SwarmEmitter,
    Trauma(0.04),
    Explosion::Small,
    DespawnTweenFinish,
    FaceVelocity,
    ShotLimit(1)
)]
pub struct Scout;

pub fn triple(center_point: Vec2) -> Formation {
    const NUM_SWARM: usize = 4;
    const SWARM_GAP: f32 = 20.;
    const SWARM_OFFSET: f32 = 20.;

    fn apply_animation(
        scout: &mut EntityCommands,
        center_point: Vec2,
        offset: f32,
        time_offset: f32,
    ) {
        let position = scout.id().into_target();
        let mut position = position.state(Vec3::new(-crate::WIDTH, crate::HEIGHT, 0.));

        scout.animation().insert(sequence((
            tween(
                Duration::from_secs_f32(3.00 + time_offset),
                EaseKind::QuadraticOut,
                position.with(bevy_tween::interpolate::translation_to(Vec3::new(
                    center_point.x + offset,
                    center_point.y,
                    0.0,
                ))),
            ),
            tween(
                Duration::from_secs(3),
                EaseKind::QuadraticIn,
                position.with(bevy_tween::interpolate::translation_to(Vec3::new(
                    crate::WIDTH,
                    crate::HEIGHT,
                    0.,
                ))),
            ),
        )));
    }

    Formation::with_velocity(Vec2::default(), move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            for i in 0..NUM_SWARM {
                let x = (i as f32 - NUM_SWARM as f32 / 2.) * SWARM_GAP;
                let y = noise::simplex_noise_2d(Vec2::new(x - SWARM_OFFSET, 0.)) * 20.;
                let x_offset = 12. * i as f32 - (NUM_SWARM as f32 - 1.) * 6.;

                let time_offset = i as f32 * 0.1;
                let mut commands = root.spawn((
                    Scout,
                    Platoon(root.target_entity()),
                    EmitterDelay::new(2.8 + time_offset),
                    Transform::from_xyz(x - x_offset - SWARM_OFFSET, y, 0.),
                ));

                apply_animation(&mut commands, center_point, x_offset, time_offset);
            }
        });
    })
}
