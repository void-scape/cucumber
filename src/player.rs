use crate::{
    DespawnRestart, GameState, HEIGHT, Layer, RES_HEIGHT, RES_WIDTH, RESOLUTION_SCALE,
    animation::{AnimationController, AnimationIndices},
    assets::{self, MISC_PATH, MiscLayout},
    bullet::{
        BulletTimer,
        emitter::{BulletModifiers, GattlingEmitter, PlayerEmitter},
    },
    end,
    health::{Dead, Health, Shield},
    minions::{Gunner, GunnerAnchor, GunnerLeader, GunnerWeapon},
    pickups::{Material, PickupEvent, Upgrade, Weapon},
};
use avian2d::prelude::*;
use bevy::{
    color::palettes::css::{DARK_RED, SKY_BLUE},
    ecs::{
        component::HookContext, entity_disabling::Disabled, system::RunSystemOnce,
        world::DeferredWorld,
    },
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::{
    interpolate::sprite_color,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
#[cfg(not(debug_assertions))]
use bevy_tween::{
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::{cmp::Ordering, f32, time::Duration};

pub const PLAYER_HEALTH: f32 = 3.0;
pub const PLAYER_SHIELD: f32 = 1.0;
const PLAYER_EASE_DUR: f32 = 1.;
pub const PLAYER_SPEED: f32 = 80.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::StartGame), spawn_player)
            .add_systems(OnEnter(GameState::Game), disable_all_emitters)
            .add_systems(
                Update,
                (
                    handle_pickups,
                    zero_rotation,
                    update_player_sprites,
                    health_effects,
                ),
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

fn spawn_player(
    mut commands: Commands,
    //mut writer: EventWriter<PickupEvent>,
    //mut mats: ResMut<MinMaterials>,
) {
    let player = commands
        .spawn((Player, Transform::from_xyz(0., -HEIGHT / 6., 0.)))
        .with_children(|root| {
            root.spawn(GattlingEmitter);
            //root.spawn((GattlingEmitter, Transform::from_xyz(2.5, 0., 0.)));
            //root.spawn((GattlingEmitter, Transform::from_xyz(-2.5, 0., 0.)));
        })
        .id();
    //commands.spawn((Miner, MinerLeader(player)));

    commands.spawn((
        Gunner,
        GunnerLeader(player),
        GunnerAnchor::Right,
        GunnerWeapon(Weapon::Bullet),
    ));
    commands.spawn((
        Gunner,
        GunnerLeader(player),
        GunnerAnchor::Left,
        GunnerWeapon(Weapon::Bullet),
    ));
    commands.spawn((
        Gunner,
        GunnerLeader(player),
        GunnerAnchor::Bottom,
        GunnerWeapon(Weapon::Bullet),
    ));

    let dur = Duration::from_secs_f32(PLAYER_EASE_DUR);
    run_after(
        dur,
        move |mut commands: Commands| {
            commands.entity(player).remove::<BlockControls>();
        },
        &mut commands,
    );

    //writer.write(PickupEvent::Weapon(Weapon::Bullet));
    //writer.write(PickupEvent::Weapon(Weapon::Bullet));
    //
    //if crate::SKIP_WAVES {
    //    writer.write(PickupEvent::Weapon(Weapon::Bullet));
    //    writer.write(PickupEvent::Weapon(Weapon::Bullet));
    //    writer.write(PickupEvent::Weapon(Weapon::Missile));
    //    writer.write(PickupEvent::Upgrade(Upgrade::Speed(0.2)));
    //    writer.write(PickupEvent::Upgrade(Upgrade::Speed(0.2)));
    //    writer.write(PickupEvent::Upgrade(Upgrade::Juice(0.2)));
    //    for _ in 0..5 {
    //        mats.0 *= 2;
    //    }
    //}

    #[cfg(not(debug_assertions))]
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

#[derive(Component)]
#[require(
    Transform,
    LinearVelocity,
    Shield::full(PLAYER_SHIELD),
    Health::full(PLAYER_HEALTH),
    RigidBody::Dynamic,
    Collider::rectangle(2., 2.),
    CollidingEntities,
    CollisionLayers::new(Layer::Player, [Layer::Bounds, Layer::Bullet, Layer::Collectable]),
    BulletModifiers,
    Materials,
    DespawnRestart,
)]
#[component(on_add = Self::on_add)]
pub struct Player;

impl Player {
    //pub fn bullet_emitter() -> impl Bundle {
    //    (
    //        DualEmitter::enemy(3.),
    //        BulletModifiers {
    //            damage: 0.5,
    //            rate: Rate::Factor(2.),
    //            speed: 1.5,
    //        },
    //        Polarity::North,
    //    )
    //}
    //
    //pub fn missile_emitter() -> impl Bundle {
    //    (HomingEmitter::<Enemy>::enemy(), Polarity::North)
    //}
    //
    //pub fn laser_emitter() -> impl Bundle {
    //    (
    //        LaserEmitter::enemy(),
    //        BulletModifiers {
    //            damage: 0.2,
    //            ..Default::default()
    //        },
    //        Polarity::North,
    //    )
    //}
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

#[derive(Component)]
pub struct WeaponEntity(pub Entity);

impl Player {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().queue(move |world: &mut World| {
            world
                .run_system_once(
                    move |mut commands: Commands,
                          server: Res<AssetServer>,
                          misc_layout: Res<MiscLayout>| {
                        let mut actions = Actions::<AliveContext>::default();
                        actions.bind::<MoveAction>().to((
                            Cardinal::wasd_keys(),
                            Cardinal::arrow_keys(),
                            Cardinal::dpad_buttons(),
                            Axial::left_stick().with_modifiers_each(
                                DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15),
                            ),
                        ));

                        actions
                            .bind::<ShootAction>()
                            .to((KeyCode::Space, GamepadButton::RightTrigger));

                        actions
                            .bind::<SwitchGunAction>()
                            .to((KeyCode::ShiftLeft, GamepadButton::South));

                        commands.entity(ctx.entity).insert((
                            actions,
                            assets::sprite_rect8(&server, assets::SHIPS_PATH, UVec2::new(1, 4)),
                            BulletTimer {
                                timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
                            },
                        ));

                        commands.entity(ctx.entity).with_child((
                            PlayerBlasters,
                            Visibility::Hidden,
                            Transform::from_xyz(0., -7., -1.),
                            Sprite::from_atlas_image(
                                server.load(MISC_PATH),
                                TextureAtlas::from(misc_layout.0.clone()),
                            ),
                            AnimationController::from_seconds(
                                AnimationIndices::repeating(18..=21),
                                0.1,
                            ),
                        ));
                    },
                )
                .unwrap();
        });
    }
}

#[derive(InputContext)]
pub struct AliveContext;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

// TODO: make this for enemies too?
#[derive(Component)]
struct PlayerBlasters;

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
struct ShootAction;

#[derive(Component)]
struct MGSounds;

fn enable_emitters(
    _: Trigger<Started<ShootAction>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut emitters: Query<(Entity, &mut BulletTimer), (With<PlayerEmitter>, With<Disabled>)>,
) {
    for (entity, mut timer) in emitters.iter_mut() {
        commands.entity(entity).remove::<Disabled>();
        let duration = timer.timer.duration();
        timer.timer.set_elapsed(duration);
    }

    commands.spawn((
        MGSounds,
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
    emitters: Query<Entity, With<PlayerEmitter>>,
    sound: Single<Entity, With<MGSounds>>,
) {
    commands.entity(*sound).despawn();
    for entity in emitters.iter() {
        commands.entity(entity).insert(Disabled);
    }
}

fn disable_all_emitters(mut commands: Commands, emitters: Query<Entity, With<PlayerEmitter>>) {
    for entity in emitters.iter() {
        commands.entity(entity).insert(Disabled);
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
struct SwitchGunAction;

fn switch_emitters(
    _: Trigger<Started<SwitchGunAction>>,
    //mut commands: Commands,
    //player: Single<&Actions<AliveContext>)>,
    mut gunners: Query<&mut GunnerWeapon, With<Gunner>>,
    mut next_weapon: Local<Weapon>,
) {
    //let (actions, gunners) = player.into_inner();

    for mut weapon in gunners.iter_mut() {
        weapon.0 = *next_weapon;
    }

    match *next_weapon {
        Weapon::Bullet => *next_weapon = Weapon::Missile,
        Weapon::Missile => *next_weapon = Weapon::Bullet,
        _ => {}
    }
}

fn update_player_sprites(
    player: Single<(&LinearVelocity, &mut Sprite), (With<Player>, Changed<LinearVelocity>)>,
    mut blasters: Single<&mut Visibility, With<PlayerBlasters>>,
) {
    let (velocity, mut sprite) = player.into_inner();

    let (tl, br) = match velocity.0.x.total_cmp(&0.) {
        Ordering::Less => (Vec2::new(0., 4.), Vec2::new(1., 5.)),
        Ordering::Greater => (Vec2::new(2., 4.), Vec2::new(3., 5.)),
        Ordering::Equal => (Vec2::new(1., 4.), Vec2::new(2., 5.)),
    };

    sprite.rect = Some(Rect::from_corners(tl * 8., br * 8.));

    if velocity.0.y > f32::EPSILON {
        **blasters = Visibility::Visible;
    } else {
        **blasters = Visibility::Hidden;
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
}

fn handle_pickups(
    mut commands: Commands,
    q: Single<
        (
            Entity,
            //&mut WeaponEntity,
            &mut BulletModifiers,
            &mut Materials,
            &mut Shield,
        ),
        With<Player>,
    >,
    mut events: EventReader<PickupEvent>,
) {
    let (player, mut mods, mut materials, mut shield) = q.into_inner();
    for event in events.read() {
        match event {
            PickupEvent::Weapon(weapon) => {
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

fn health_effects(mut commands: Commands, player: Single<(Ref<Shield>, Ref<Health>)>) {
    let (shield, health) = player.into_inner();

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
}
