use super::Enemy;
use super::LowHealthEffects;
use super::formation::Formation;
use super::formation::Platoon;
use super::formation::animate_entrance;
use crate::bullet::RedOrb;
use crate::bullet::emitter::EmitterBullet;
use crate::bullet::emitter::EmitterDelay;
use crate::bullet::emitter::EmitterSample;
use crate::bullet::emitter::ORB_SPEED;
use crate::bullet::emitter::PulseTime;
use crate::bullet::emitter::PulseTimer;
use crate::bullet::emitter::Rate;
use crate::bullet::emitter::SpiralOffset;
use crate::{bullet::emitter::BulletModifiers, effects::Explosion, health::Health};
use avian2d::prelude::*;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

#[derive(Default, Clone, Copy, Component)]
#[require(
    Enemy,
    Collider::circle(21.),
    Health::full(80.),
    LowHealthEffects,
    VergerEmitter,
    Explosion::Big,
    DebugCircle::color(21., YELLOW)
)]
pub struct Verger;

pub fn verger() -> Formation {
    Formation::with_velocity(
        Vec2::ZERO,
        |formation: &mut EntityCommands, server: &AssetServer| {
            let id = formation.id();
            animate_entrance(
                server,
                &mut formation.commands(),
                (Verger, ChildOf(id), Platoon(id)),
                None,
                1.5,
                Vec3::new(0., 16., 0.),
                Vec3::new(-40., -32., 0.),
                Quat::default(),
                Quat::default(),
            );
        },
    )
}

#[derive(Default, Clone, Copy, Component)]
#[require(
    Transform,
    BulletModifiers,
    SpiralOffset,
    PulseTimer::new(Rate::Secs(2.), 3.0, 0.1, 10)
)]
pub struct VergerEmitter;

impl VergerEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                &mut PulseTimer,
                &BulletModifiers,
                &ChildOf,
                &GlobalTransform,
            ),
            (With<VergerEmitter>, Without<EmitterDelay>),
        >,
        parents: Query<Option<&BulletModifiers>>,
        time: Res<Time>,
        mut writer: EventWriter<EmitterSample>,
        mut commands: Commands,
    ) {
        for (mut timer, mods, parent, transform) in emitters.iter_mut() {
            let Ok(parent_mods) = parents.get(parent.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            if !timer.just_finished(&time) {
                continue;
            }

            let bullets = 6;
            let new_transform = transform.compute_transform();
            let mut angle_offset = timer.current_pulse() as f32 * std::f32::consts::TAU
                / (bullets * timer.pulses()) as f32;
            if timer.current_pulse() % 2 == 0 {
                angle_offset *= -1.;
            }

            // let speed = ORB_SPEED * mods.speed * (1. - timer.current_pulse() as f32 * 0.05);
            let speed = ORB_SPEED * mods.speed;
            for angle in 0..bullets {
                let angle = (angle as f32 / bullets as f32) * std::f32::consts::TAU
                    // + timer.current_pulse() as f32 * std::f32::consts::PI * 0.01
                    + angle_offset;

                commands.spawn((
                    RedOrb,
                    LinearVelocity(Vec2::from_angle(angle) * speed),
                    new_transform,
                ));
            }

            writer.write(EmitterSample(EmitterBullet::Orb));
        }
    }
}
