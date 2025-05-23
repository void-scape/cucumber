use super::Enemy;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use crate::boss::gradius::emitters::SpiralOffset;
use crate::bullet::RedOrb;
use crate::bullet::emitter::BulletCommands;
use crate::bullet::emitter::BulletSpeed;
use crate::bullet::emitter::Emitter;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterCtx;
use crate::bullet::emitter::ORB_SPEED;
use crate::bullet::emitter::PulseTime;
use crate::bullet::emitter::PulseTimer;
use crate::bullet::emitter::Rate;
use crate::bullet::emitter::ShootEmitter;
use crate::{bullet::emitter::BulletModifiers, effects::Explosion, health::Health};
use avian2d::prelude::*;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::circle(21.),
    Health::full(100.),
    LowHealthEffects,
    VergerEmitter,
    Explosion::Big,
    DebugCircle::color(21., YELLOW)
)]
pub struct Verger;

pub fn verger(position: Vec2) -> Formation {
    Formation::with_velocity(
        Vec2::ZERO,
        move |formation: &mut EntityCommands, server: &AssetServer| {
            let id = formation.id();
            animate_entrance(
                server,
                &mut formation.commands(),
                (Verger, ChildOf(id), Platoon(id)),
                None,
                1.5,
                Vec3::new(0., 16., 0.),
                position.extend(0.),
                Quat::default(),
                Quat::default(),
            );
        },
    )
}

#[derive(Default, Clone, Copy, Component)]
#[require(Transform, Emitter, SpiralOffset, BulletSpeed::new(ORB_SPEED))]
pub struct VergerEmitter;

impl ShootEmitter for VergerEmitter {
    type Timer = PulseTimer;

    fn timer(&self, _mods: &BulletModifiers) -> Self::Timer {
        PulseTimer::new(Rate::Secs(2.), 3.0, 0.1, 10)
    }

    fn spawn_bullets(
        &self,
        mut commands: BulletCommands,
        transform: Transform,
        ctx: EmitterCtx<Self::Timer>,
    ) {
        let bullets = 6;
        let mut angle_offset = ctx.timer.current_pulse() as f32 * std::f32::consts::TAU
            / (bullets * ctx.timer.pulses()) as f32;
        if ctx.timer.current_pulse() % 2 == 0 {
            angle_offset *= -1.;
        }

        for angle in 0..bullets {
            let angle = (angle as f32 / bullets as f32) * std::f32::consts::TAU + angle_offset;
            commands.spawn_angled(angle, (RedOrb, transform));
        }
    }

    fn sample() -> Option<EmitterBullet> {
        Some(EmitterBullet::Orb)
    }
}
