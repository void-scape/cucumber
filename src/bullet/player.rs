use super::{BasicBullet, Bullet, Missile, PlayerBullet, emitter::*};
use crate::Layer;
use crate::health::Damage;
use crate::particles::{self, *};
use crate::player::{PowerUps, ShotKind};
use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Default, Component)]
#[require(
    Transform,
    BulletModifiers,
    EmitterState,
    ParticleBundle<EmitterState> = Self::particles(),
)]
#[component(on_add = Self::insert_timer)]
pub struct PlayerGattlingEmitter;

impl PlayerGattlingEmitter {
    fn particles() -> ParticleBundle<EmitterState> {
        ParticleBundle::<EmitterState>::from_emitter(
            ParticleEmitter::from_effect("particles/bullet_shells.ron")
                .with_sprite("shell.png")
                .with(particles::transform(
                    Transform::from_xyz(0., -5., -1.).with_rotation(Quat::from_rotation_z(PI)),
                )),
        )
    }

    fn insert_timer(mut world: DeferredWorld, ctx: HookContext) {
        let mods = world.get::<BulletModifiers>(ctx.entity).unwrap();
        let duration = mods.rate.duration(PLAYER_BULLET_RATE);
        let mut timer = PulseTimer::new(Rate::Secs(duration.as_secs_f32()), 0.3, 0.15, 2);
        timer.reset_active();
        world.commands().entity(ctx.entity).insert(timer);
    }
}

impl PlayerGattlingEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                &EmitterState,
                &mut PulseTimer,
                &BulletModifiers,
                &GlobalTransform,
                &ChildOf,
            ),
            With<PlayerGattlingEmitter>,
        >,
        parents: Query<Option<&BulletModifiers>, With<Children>>,
        time: Res<Time>,
        power: Option<Res<PowerUps>>,
        mut commands: Commands,
    ) {
        let Some(power) = power else {
            return;
        };

        for (state, mut timer, mods, transform, child_of) in emitters.iter_mut() {
            if !state.enabled {
                continue;
            }

            timer.tick(&time);
            if !timer.just_finished() {
                continue;
            }

            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += Vec3::Y * 1.0;

            spawn_normal_bullets(
                &mut commands,
                &mut timer,
                ShotKind::Normal,
                new_transform,
                &mods,
                power.get(),
            );
        }
    }
}

fn spawn_normal_bullets(
    commands: &mut Commands,
    timer: &mut PulseTimer,
    kind: ShotKind,
    transform: Transform,
    mods: &BulletModifiers,
    power: usize,
) {
    match power {
        0 => {
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
        }
        1 => {
            timer.wait.set_duration(Duration::from_secs_f32(0.2));
            let angle = 0.1;
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 2.5))
                    .with_rotation(Quat::from_rotation_z(-angle)),
                (Vec2::Y + Vec2::new(angle, 0.)).normalize() * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform,
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 2.5))
                    .with_rotation(Quat::from_rotation_z(angle)),
                (Vec2::Y - Vec2::new(angle, 0.)).normalize() * PLAYER_BULLET_SPEED,
            );
        }
        2 => {
            timer.wait.set_duration(Duration::from_secs_f32(0.1));
            let angle = 0.1;
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 7.5))
                    .with_rotation(Quat::from_rotation_z(-2. * angle)),
                (Vec2::Y + Vec2::new(2. * angle, 0.)).normalize() * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 5.))
                    .with_rotation(Quat::from_rotation_z(-angle)),
                (Vec2::Y + Vec2::new(angle, 0.)).normalize() * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 5.))
                    .with_rotation(Quat::from_rotation_z(angle)),
                (Vec2::Y - Vec2::new(angle, 0.)).normalize() * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 7.5))
                    .with_rotation(Quat::from_rotation_z(2. * angle)),
                (Vec2::Y - Vec2::new(2. * angle, 0.)).normalize() * PLAYER_BULLET_SPEED,
            );
        }
        _ => error!("invalid power level: {}", power),
    }
}

#[derive(Default, Component)]
#[require(
    Transform,
    BulletModifiers,
    EmitterState,
    ParticleBundle<EmitterState> = Self::particles(),
)]
#[component(on_add = Self::insert_timer)]
pub struct PlayerFocusEmitter;

impl PlayerFocusEmitter {
    fn particles() -> ParticleBundle<EmitterState> {
        ParticleBundle::<EmitterState>::from_emitter(
            ParticleEmitter::from_effect("particles/bullet_shells.ron")
                .with_sprite("missile_shell.png")
                .with(particles::transform(
                    Transform::from_xyz(0., -5., -1.).with_rotation(Quat::from_rotation_z(PI)),
                )),
        )
    }

    fn insert_timer(mut world: DeferredWorld, ctx: HookContext) {
        let mods = world.get::<BulletModifiers>(ctx.entity).unwrap();
        let duration = mods.rate.duration(PLAYER_BULLET_RATE);
        let mut timer = PulseTimer::new(Rate::Secs(duration.as_secs_f32()), 0.3, 0.15, 2);
        timer.reset_active();
        world.commands().entity(ctx.entity).insert(timer);
    }
}

impl PlayerFocusEmitter {
    pub fn shoot_bullets(
        mut emitters: Query<
            (
                &EmitterState,
                &mut PulseTimer,
                &BulletModifiers,
                &GlobalTransform,
                &ChildOf,
            ),
            With<PlayerFocusEmitter>,
        >,
        parents: Query<Option<&BulletModifiers>, With<Children>>,
        time: Res<Time>,
        power: Option<Res<PowerUps>>,
        mut commands: Commands,
    ) {
        let Some(power) = power else {
            return;
        };

        for (state, mut timer, mods, transform, child_of) in emitters.iter_mut() {
            if !state.enabled {
                continue;
            }

            timer.tick(&time);
            if !timer.just_finished() {
                continue;
            }

            let Ok(parent_mods) = parents.get(child_of.parent()) else {
                continue;
            };
            let mods = parent_mods.map(|m| m.join(mods)).unwrap_or(*mods);

            let mut new_transform = transform.compute_transform();
            new_transform.translation += Vec3::Y * 1.0;

            spawn_focus_bullets(
                &mut commands,
                &mut timer,
                ShotKind::Focus,
                new_transform,
                &mods,
                power.get(),
            );
        }
    }
}

fn spawn_focus_bullets(
    commands: &mut Commands,
    timer: &mut PulseTimer,
    kind: ShotKind,
    transform: Transform,
    mods: &BulletModifiers,
    power: usize,
) {
    match power {
        0 => {
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
        }
        1 => {
            timer.wait.set_duration(Duration::from_secs_f32(0.2));
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform,
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
        }
        2 => {
            timer.wait.set_duration(Duration::from_secs_f32(0.1));
            spawn_bullet(
                commands,
                kind,
                mods,
                transform.with_translation(transform.translation + Vec3::new(7.5, -3., 0.)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform.with_translation(transform.translation + Vec3::new(5., -2., 0.)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x - 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform
                    .with_translation(transform.translation.with_x(transform.translation.x + 2.5)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform.with_translation(transform.translation + Vec3::new(-5., -2., 0.)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
            spawn_bullet(
                commands,
                kind,
                mods,
                transform.with_translation(transform.translation + Vec3::new(-7.5, -3., 0.)),
                Vec2::Y * PLAYER_BULLET_SPEED,
            );
        }
        _ => error!("invalid power level: {}", power),
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    kind: ShotKind,
    _mods: &BulletModifiers,
    mut transform: Transform,
    velocity: Vec2,
) {
    let mut commands = match kind {
        ShotKind::Normal => commands.spawn(BasicBullet),
        ShotKind::Focus => {
            transform.rotate_z(PI / 4.);
            commands.spawn(Missile)
        }
    };
    commands.insert((
        PlayerBullet,
        LinearVelocity(velocity),
        transform,
        Bullet::target_layer(Layer::Enemy),
        Damage::new(1.),
    ));
}
