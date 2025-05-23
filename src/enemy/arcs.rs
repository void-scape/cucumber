use super::Enemy;
use super::Trauma;
use super::formation::Formation;
use super::formation::Platoon;
use crate::bullet::BasicBullet;
use crate::bullet::BulletTimer;
use crate::bullet::emitter::BulletCommands;
use crate::bullet::emitter::Emitter;
use crate::bullet::emitter::EmitterCtx;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::RotateBullet;
use crate::bullet::emitter::ShootEmitter;
use crate::bullet::emitter::Target;
use crate::enemy::FaceVelocity;
use crate::tween::DespawnTweenFinish;
use crate::{
    auto_collider::ImageCollider,
    bullet::emitter::{BulletModifiers, Rate},
    effects::Explosion,
    health::Health,
};
use avian2d::prelude::Collider;
use bevy::color::palettes::css::GRAY;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use bevy_tween::interpolate::translation;
use bevy_tween::{
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::time::Duration;

const BULLET_SPEED: f32 = 50.;

#[derive(Default, Component)]
#[require(
    Enemy,
    ArcsEmitter,
    Health::full(50.),
    DebugCircle::color(12., GRAY),
    Collider::circle(12.),
    BulletModifiers {
        rate: Rate::Factor(0.2),
        ..Default::default()
    },
    Trauma(0.04),
    Explosion::Small,
    FaceVelocity,
)]
pub struct Arcs;

// This one pivoted halfway through
pub fn persistent() -> Formation {
    Formation::new(move |formation: &mut EntityCommands, _| {
        formation.with_children(|root| {
            let platoon = root.target_entity();

            root.spawn((Arcs, Platoon(platoon), EmitterDelay::new(1.)));
        });
    })
}

// NOTE: we assume this is always on an enemy
#[derive(Component, Default)]
#[require(Transform, Emitter, Target::player())]
pub struct ArcsEmitter;

impl ShootEmitter for ArcsEmitter {
    type Timer = BulletTimer;

    fn timer(&self, _mods: &BulletModifiers) -> Self::Timer {
        BulletTimer::ready(3.)
    }

    fn spawn_bullets(
        &self,
        mut commands: BulletCommands,
        transform: Transform,
        ctx: EmitterCtx<Self::Timer>,
    ) {
        let intended_speed = BULLET_SPEED * ctx.mods.speed;
        let distance = ctx.player_position.distance(transform.translation.xy());
        let duration = Duration::from_secs_f32(distance / intended_speed);

        let bullet = commands
            .spawn_naked(BasicBullet)
            .look_at_offset(ctx.target, -std::f32::consts::FRAC_PI_2)
            .id();

        commands
            .commands
            .entity(bullet)
            .animation()
            .insert_tween_here(
                duration,
                EaseKind::QuadraticOut,
                bullet.into_target().with(translation(
                    transform.translation,
                    ctx.player_position.extend(transform.translation.z),
                )),
            );
    }
}
