use crate::{
    Avian, DespawnRestart, GameState, HEIGHT, Layer, RES_HEIGHT, RES_WIDTH, RESOLUTION_SCALE,
    bullet::{
        BulletTimer,
        emitter::{BulletModifiers, EmitterState, GattlingEmitter},
    },
    effects::{Blasters, Explosion},
    end,
    enemy::Enemy,
    health::{DamageEvent, Dead, Health, HealthSet, Invincible, Shield},
    minions::{Gunner, GunnerWeapon},
    pickups::{Material, PickupEvent, Upgrade, Weapon},
    sprites::{CellSize, TiltSprite},
    tween::{OnEnd, TimeMult, time_mult},
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{BLUE, DARK_RED, RED, SKY_BLUE, WHITE},
    ecs::{component::HookContext, system::RunSystemOnce, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_optix::{
    glitch::{GlitchIntensity, GlitchSettings, glitch_intensity},
    pixel_perfect::OuterCamera,
    post_process::PostProcessCommand,
    shake::TraumaCommands,
};
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::{
    interpolate::sprite_color,
    prelude::{AnimationBuilderExt, EaseKind, Repeat, RepeatStyle},
    tween::{IntoTarget, TargetResource},
};
use std::{
    cmp::Ordering,
    f32::{self},
    time::Duration,
};

pub const PLAYER_HEALTH: f32 = 3.0;
pub const PLAYER_SHIELD: f32 = 1.0;
const PLAYER_EASE_DUR: f32 = 1.;
pub const PLAYER_SPEED: f32 = 80.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PowerUpEvent>()
            .insert_resource(WeaponRack::default())
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(OnEnter(GameState::StartGame), spawn_player)
            .add_systems(
                Update,
                (
                    zero_rotation,
                    (handle_pickups, handle_powerups, enemy_collision)
                        .run_if(in_state(GameState::Game)),
                ),
            )
            .add_systems(
                Avian,
                (handle_damage, health_effects)
                    .after(HealthSet)
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(First, handle_death)
            .add_input_context::<AliveContext>()
            .add_observer(apply_movement)
            .add_observer(stop_movement)
            .add_observer(enable_emitters)
            .add_observer(disable_emitters)
            .add_observer(switch_emitters);
    }
}

fn spawn_player(mut commands: Commands) {
    commands.insert_resource(PowerUps::default());

    let player = commands
        .spawn((
            Player,
            Transform::from_xyz(0., -HEIGHT / 6., 0.),
            Blasters(const { &[Vec3::new(0., -6., -1.)] }),
        ))
        .with_child((
            GattlingEmitter::default(),
            PlayerEmitter,
            EmitterState { enabled: false },
        ))
        .id();

    let dur = Duration::from_secs_f32(PLAYER_EASE_DUR);
    run_after(
        dur,
        move |mut commands: Commands| {
            commands.entity(player).remove::<BlockControls>();
        },
        &mut commands,
    );

    #[cfg(not(debug_assertions))]
    {
        use bevy_tween::interpolate::translation;
        commands
            .entity(player)
            .insert(BlockControls)
            .animation()
            .insert_tween_here(
                dur,
                EaseKind::SineOut,
                player.into_target().with(translation(
                    Vec3::new(0., -HEIGHT / 2. + 16., 0.),
                    Vec3::new(0., -HEIGHT / 6., 0.),
                )),
            );
    }
}

fn enemy_collision(
    mut writer: EventWriter<DamageEvent>,
    player: Single<(Entity, &CollidingEntities), With<Player>>,
    enemies: Query<&Enemy>,
) {
    let (entity, collisions) = player.into_inner();
    if enemies.iter_many(collisions.iter()).count() > 0 {
        writer.write(DamageEvent { entity, damage: 1. });
    }
}

#[derive(Event)]
pub struct PowerUpEvent;

#[derive(Default, Resource)]
struct PowerUps(usize);

fn handle_powerups(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut power_ups: ResMut<PowerUps>,
    mut reader: EventReader<PowerUpEvent>,
    mut player: Single<&mut BulletModifiers, With<Player>>,
) {
    for _ in reader.read() {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/ring.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));

        power_ups.0 += 1;

        match power_ups.0 {
            0 => unreachable!(),
            1..=4 => {
                player.damage += 0.1;
            }
            _ => error!("power up [{}] not handled", power_ups.0),
        }
    }
}

#[derive(Component)]
#[require(
    Transform,
    LinearVelocity,
    Shield::full(PLAYER_SHIELD),
    Health::full(PLAYER_HEALTH),
    RigidBody::Dynamic,
    Collider::rectangle(2., 2.),
    CollidingEntities,
    BulletModifiers,
    Materials,
    DespawnRestart,
    CollisionLayers = Self::layers(),
    Explosion::Big,
)]
#[component(on_add = Self::on_add)]
pub struct Player;

impl Player {
    fn layers() -> CollisionLayers {
        CollisionLayers::new(
            Layer::Player,
            [
                Layer::Bounds,
                Layer::Bullet,
                Layer::Collectable,
                Layer::Enemy,
            ],
        )
    }
}

#[derive(Default, Component)]
pub struct Materials(usize);

impl Materials {
    pub fn get(&self) -> usize {
        self.0
    }

    pub fn sub(&mut self, n: usize) {
        self.0 = self.0.saturating_sub(n);
    }
}

impl Player {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().queue(move |world: &mut World| {
            world
                .run_system_once(move |mut commands: Commands, server: Res<AssetServer>| {
                    let mut actions = Actions::<AliveContext>::default();
                    actions.bind::<MoveAction>().to((
                        Cardinal::wasd_keys(),
                        Cardinal::arrow_keys(),
                        Cardinal::dpad_buttons(),
                        Axial::left_stick().with_modifiers_each(
                            DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15),
                        ),
                    ));

                    actions.bind::<ShootAction>().to((
                        KeyCode::Space,
                        GamepadButton::RightTrigger,
                        GamepadButton::RightTrigger2,
                    ));

                    actions
                        .bind::<SwitchGunAction>()
                        .to((KeyCode::ShiftLeft, GamepadButton::South));

                    //    Ordering::Less => (Vec2::new(0., 5.), Vec2::new(1., 6.)),
                    //    Ordering::Greater => (Vec2::new(2., 5.), Vec2::new(3., 6.)),
                    //    Ordering::Equal => (Vec2::new(1., 5.), Vec2::new(2., 6.)),
                    //};
                    //
                    //sprite.rect = Some(Rect::from_corners(tl * 8., br * 8.));

                    commands.entity(ctx.entity).insert((
                        actions,
                        TiltSprite {
                            path: "ships.png",
                            size: CellSize::Eight,
                            //
                            left: UVec2::new(0, 0),
                            center: UVec2::new(1, 0),
                            right: UVec2::new(2, 0),
                        },
                        BulletTimer {
                            timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
                        },
                    ));
                })
                .unwrap();
        });
    }
}

#[derive(InputContext)]
pub struct AliveContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

#[derive(Component)]
pub struct BlockControls;

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    player: Single<(&mut LinearVelocity, Option<&BlockControls>), With<Player>>,
) {
    let (mut velocity, blocked) = player.into_inner();

    if blocked.is_none() {
        velocity.0 = trigger.value.clamp_length(0., 1.) * PLAYER_SPEED;
    }

    if velocity.0.x != 0.0 && velocity.0.x.abs() < f32::EPSILON {
        velocity.0.x = 0.;
    }
}

fn stop_movement(
    _: Trigger<Completed<MoveAction>>,
    mut velocity: Single<&mut LinearVelocity, (With<Player>, Without<BlockControls>)>,
) {
    velocity.0 = Vec2::default();
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
pub struct ShootAction;

#[derive(Component)]
struct MGEffects;

#[derive(Component)]
struct MGSound;

#[derive(Default, Component)]
pub struct PlayerEmitter;

fn enable_emitters(
    _: Trigger<Started<ShootAction>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    player_emitter: Single<(&mut EmitterState, &mut BulletTimer), With<PlayerEmitter>>,
    mut weapons: Query<&mut GunnerWeapon, With<Gunner>>,
    player: Single<Entity, With<Player>>,
) {
    let (mut state, mut timer) = player_emitter.into_inner();
    state.enabled = true;
    let duration = timer.timer.duration();
    timer.timer.set_elapsed(duration);

    for mut weapon in weapons.iter_mut() {
        weapon.enabled = true;
    }

    commands.spawn((
        MGSound,
        ChildOf(*player),
        //
        SamplePlayer::new(server.load("audio/sfx/mg.wav")),
        PlaybackSettings {
            volume: Volume::Linear(0.25),
            ..PlaybackSettings::LOOP
        },
    ));
}

fn disable_emitters(
    _: Trigger<Completed<ShootAction>>,
    mut commands: Commands,
    mut player_emitter: Single<&mut EmitterState, With<PlayerEmitter>>,
    mut weapons: Query<&mut GunnerWeapon, With<Gunner>>,
    sound: Single<Entity, With<MGSound>>,
) {
    commands.entity(*sound).despawn();
    player_emitter.enabled = false;
    for mut weapon in weapons.iter_mut() {
        weapon.enabled = false;
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct SwitchGunAction;

#[derive(Resource)]
pub struct WeaponRack {
    weapons: [(Weapon, bool); 2],
    index: Option<usize>,
}

impl Default for WeaponRack {
    fn default() -> Self {
        Self {
            weapons: [(Weapon::Bullet, false), (Weapon::Missile, false)],
            index: None,
        }
    }
}

impl WeaponRack {
    pub fn aquire(&mut self, weapon: Weapon) {
        for (w, enabled) in self.weapons.iter_mut() {
            if *w == weapon {
                *enabled = true;
                break;
            }
        }
    }

    pub fn expire(&mut self, weapon: Weapon) {
        for (w, enabled) in self.weapons.iter_mut() {
            if *w == weapon {
                *enabled = false;
                break;
            }
        }
    }

    pub fn next(&mut self) -> Option<Weapon> {
        match self.index {
            Some(index) => {
                if index >= self.weapons.len() - 1 {
                    self.index = Some(0);
                } else {
                    self.index = Some(index + 1);
                }
            }
            None => {
                self.index = Some(0);
            }
        }
        self.selection()
    }

    pub fn selection(&mut self) -> Option<Weapon> {
        if let Some(index) = self.index {
            match self
                .weapons
                .iter()
                .nth(index)
                .map(|(w, enabled)| enabled.then_some(*w))
                .unwrap()
            {
                Some(w) => return Some(w),
                None => {
                    match self
                        .weapons
                        .iter()
                        .chain(self.weapons.iter().take(index))
                        .skip(index + 1)
                        .enumerate()
                        .find_map(|(i, (w, enabled))| enabled.then_some((i, *w)))
                    {
                        Some((i, w)) => {
                            self.index = Some(i);
                            return Some(w);
                        }
                        None => {
                            self.index = None;
                            return None;
                        }
                    }
                }
            }
        }

        match self
            .weapons
            .iter()
            .enumerate()
            .find_map(|(i, (w, enabled))| enabled.then_some((i, *w)))
        {
            Some((i, w)) => {
                self.index = Some(i);
                return Some(w);
            }
            None => {
                self.index = None;
                return None;
            }
        }
    }
}

fn switch_emitters(
    _: Trigger<Started<SwitchGunAction>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    player: Single<&Actions<AliveContext>>,
    mut gunners: Query<&mut GunnerWeapon, With<Gunner>>,
    mut rack: ResMut<WeaponRack>,
) {
    let mut changed = false;
    if let Some(next) = rack.next() {
        for mut weapon in gunners.iter_mut() {
            if next != weapon.weapon {
                changed = true;
            }
            weapon.weapon = next;
            weapon.enabled = player.action::<ShootAction>().state() == ActionState::Fired;
        }
    }

    if changed {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/shotgun_rack.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.35),
                ..PlaybackSettings::ONCE
            },
        ));
    } else {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/bfxr/failed.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.7),
                ..PlaybackSettings::ONCE
            },
        ));
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/electric.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.5),
                ..PlaybackSettings::ONCE
            },
        ));
    }
}

// TODO: this does not work? we don't brush on anything anyways

/// Brushing along edges rotates the player.
///
/// Transform and all physics things are synced in [`PhysicsSet::Sync`].
fn zero_rotation(mut player: Single<&mut Transform, With<Player>>) {
    player.rotation = Quat::default();
}

fn handle_death(mut commands: Commands, player: Single<Entity, (With<Player>, With<Dead>)>) {
    commands.entity(*player).despawn();
    commands.queue(|world: &mut World| world.run_system_once(end::show_loose_screen));
    commands
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(2.),
            EaseKind::QuadraticIn,
            TargetResource.with(time_mult(1., 0.)),
        )
        .insert(DespawnRestart);
}

fn restart(mut commands: Commands, mut time: ResMut<TimeMult>) {
    commands.insert_resource(WeaponRack::default());
    time.0 = 1.;
}

fn handle_pickups(
    mut commands: Commands,
    server: Res<AssetServer>,
    q: Single<(Entity, &mut BulletModifiers, &mut Materials, &mut Shield), With<Player>>,
    mut events: EventReader<PickupEvent>,
    mut rack: ResMut<WeaponRack>,
) {
    let (player, mut mods, mut materials, mut shield) = q.into_inner();
    for event in events.read() {
        match event {
            PickupEvent::Weapon(weapon) => {
                rack.aquire(*weapon);
                commands.spawn((
                    SamplePlayer::new(server.load("audio/sfx/pickup.wav")),
                    PlaybackSettings {
                        volume: Volume::Linear(0.2),
                        ..PlaybackSettings::ONCE
                    },
                ));

                // commands.entity(weapon_entity.0).despawn();

                //let emitter = match weapon {
                //    Weapon::Bullet => commands.spawn(Player::bullet_emitter()).id(),
                //    Weapon::Missile => commands.spawn(Player::missile_emitter()).id(),
                //    Weapon::Laser => commands.spawn(Player::laser_emitter()).id(),
                //};
                //
                ////weapon_entity.0 = emitter;
                //commands.entity(player).add_child(emitter);

                //commands.run_system_cached(
                //    |player: Single<&Children, With<Player>>,
                //     mut children: Query<&mut Transform, With<BulletModifiers>>| {
                //        let total = children.iter_many(player.iter()).count() as f32;
                //
                //        let padding = Vec3::new(4.0, 0.0, 0.0);
                //        let start = padding * -0.5 * total;
                //
                //        let mut children = children.iter_many_mut(player.iter());
                //        let mut i = 0;
                //
                //        while let Some(mut transform) = children.fetch_next() {
                //            transform.translation = start + padding * i as f32;
                //
                //            i += 1;
                //        }
                //    },
                //);
            }
            PickupEvent::Upgrade(Upgrade::Speed(s)) => mods.rate.add_factor(*s),
            PickupEvent::Upgrade(Upgrade::Juice(j)) => mods.damage += *j,
            PickupEvent::Material(mat) => match mat {
                Material::Parts => materials.0 += 1,
                Material::Shield => shield.heal(1. / 10.),
            },
        }
    }
}

#[derive(Component)]
struct FlickerAnimation;

fn handle_damage(
    mut commands: Commands,
    mut reader: EventReader<DamageEvent>,
    player: Single<(Entity, Ref<Shield>, Ref<Health>), (With<Player>, Without<Invincible>)>,
) {
    let (player, shield, _health) = player.into_inner();

    if reader.read().any(|e| e.entity == player) {
        commands.entity(player).insert(Invincible);

        let color = if shield.is_changed() && shield.empty() {
            BLUE.into()
        } else {
            RED.into()
        };
        let flicker = commands
            .animation()
            .repeat(Repeat::times(3))
            .repeat_style(RepeatStyle::PingPong)
            .insert_tween_here(
                Duration::from_secs_f32(0.25),
                EaseKind::Linear,
                player.into_target().with(sprite_color(WHITE.into(), color)),
            )
            .insert(FlickerAnimation)
            .id();
        commands.entity(player).add_child(flicker);

        run_after(
            Duration::from_secs_f32(1.5),
            |mut commands: Commands,
             mut player: Single<(Entity, &mut Sprite), With<Player>>,
             animation: Single<Entity, With<FlickerAnimation>>| {
                commands.entity(*animation).despawn();
                commands.entity(player.0).remove::<Invincible>();
                player.1.color = WHITE.into();
            },
            &mut commands,
        );
    }
}

fn health_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<DamageEvent>,
    player: Single<(Entity, Ref<Shield>, Ref<Health>), (With<Player>, Without<Invincible>)>,
    camera: Single<Entity, With<OuterCamera>>,
) {
    let (player, shield, health) = player.into_inner();

    if shield.is_changed() && shield.empty() {
        let mask = commands
            .spawn((
                DespawnRestart,
                Sprite::from_color(
                    SKY_BLUE.with_alpha(0.2),
                    Vec2::new(RES_WIDTH * RESOLUTION_SCALE, RES_HEIGHT * RESOLUTION_SCALE),
                ),
                Transform::from_xyz(0., 0., 999.),
            ))
            .id();
        commands.entity(mask).animation().insert_tween_here(
            Duration::from_secs_f32(0.1),
            EaseKind::BounceIn,
            mask.into_target().with(sprite_color(
                SKY_BLUE.with_alpha(0.2).into(),
                SKY_BLUE.with_alpha(0.).into(),
            )),
        );
    }

    if health.is_changed() && health.current() != health.max() {
        let mask = commands
            .spawn((
                DespawnRestart,
                Sprite::from_color(
                    DARK_RED.with_alpha(0.2),
                    Vec2::new(RES_WIDTH * RESOLUTION_SCALE, RES_HEIGHT * RESOLUTION_SCALE),
                ),
                Transform::from_xyz(0., 0., 999.),
            ))
            .id();
        commands.entity(mask).animation().insert_tween_here(
            Duration::from_secs_f32(0.3),
            EaseKind::BounceIn,
            mask.into_target().with(sprite_color(
                DARK_RED.with_alpha(0.2).into(),
                DARK_RED.with_alpha(0.).into(),
            )),
        );
    }

    if reader.read().any(|e| e.entity == player) {
        let on_end = OnEnd::new(&mut commands, |mut commands: Commands| {
            commands.remove_post_process::<GlitchSettings, OuterCamera>();
            commands.remove_post_process::<GlitchIntensity, OuterCamera>();
        });

        commands.post_process::<OuterCamera>(GlitchSettings::default());
        commands.post_process::<OuterCamera>(GlitchIntensity::default());
        commands
            .animation()
            .insert_tween_here(
                Duration::from_secs_f32(0.4),
                EaseKind::Linear,
                camera.into_target().with(glitch_intensity(0.3, 0.0)),
            )
            .insert(on_end);

        commands.add_trauma(0.15);
        //commands
        //    .animation()
        //    .insert_tween_here(
        //        Duration::from_secs_f32(0.25),
        //        EaseKind::Linear,
        //        TargetResource.with(time_mult(0.25, 1.)),
        //    )
        //    .insert(DespawnRestart);

        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/melee.wav")),
            PitchRange(0.98..1.02),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/player_damage.wav")),
            //PitchRange(0.98..1.02),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));
    }
}
