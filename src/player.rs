use crate::{
    GameState, HEIGHT, Layer,
    animation::{AnimationController, AnimationIndices},
    assets::{self, MISC_PATH, MiscLayout},
    auto_collider::ImageCollider,
    bullet::{
        BulletCollisionEvent, BulletRate, BulletSource, BulletTimer, Polarity,
        emitter::{DualEmitter, HomingEmitter},
    },
    enemy::EnemyType,
    health::{Dead, Health},
    pickups::{PickupEvent, Upgrade, Weapon},
};
use avian2d::prelude::*;
use bevy::{
    ecs::{component::HookContext, system::RunSystemOnce, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_tween::{
    combinator::tween,
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::{cmp::Ordering, f32, time::Duration};

pub const PLAYER_HEALTH: f32 = 5.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), |mut commands: Commands| {
            let starting_weapon = commands
                .spawn((DualEmitter::enemy(3.), Polarity::North))
                .id();

            let player = commands
                .spawn((Player, WeaponEntity(starting_weapon), BlockControls))
                .add_child(starting_weapon)
                .id();
            let dur = Duration::from_secs_f32(1.);
            run_after(
                dur,
                move |mut commands: Commands| {
                    commands.entity(player).remove::<BlockControls>();
                },
                &mut commands,
            );
            commands.animation().insert(tween(
                dur,
                EaseKind::SineOut,
                player.into_target().with(translation(
                    Vec3::new(0., -HEIGHT / 2. + 16., 0.),
                    Vec3::new(0., -HEIGHT / 6., 0.),
                )),
            ));
        })
        .add_systems(
            Update,
            (handle_pickups, damage_effects, handle_death, zero_rotation),
        )
        .add_input_context::<AliveContext>()
        .add_observer(apply_movement)
        .add_observer(stop_movement);
    }
}

#[derive(Component)]
#[require(
    Transform,
    LinearVelocity,
    Health::full(PLAYER_HEALTH),
    ImageCollider,
    RigidBody::Dynamic,
    CollidingEntities,
    CollisionLayers = collision_layers(),
    BulletRate,
)]
#[component(on_add = Self::on_add)]
pub struct Player;

fn collision_layers() -> CollisionLayers {
    CollisionLayers::new([Layer::Player], [Layer::Bounds, Layer::Bullet])
}

#[derive(Component)]
struct WeaponEntity(Entity);

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

// TODO: make this for enemies too?
#[derive(Component)]
struct PlayerBlasters;

#[derive(Component)]
struct BlockControls;

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    player: Single<(&mut LinearVelocity, &mut Sprite), (With<Player>, Without<BlockControls>)>,
    mut blasters: Single<&mut Visibility, With<PlayerBlasters>>,
) {
    let (mut velocity, mut sprite) = player.into_inner();

    velocity.0 = trigger.value.normalize_or_zero() * 60.;
    if velocity.0.x.abs() < f32::EPSILON {
        velocity.0.x = 0.;
    }

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

fn stop_movement(
    _: Trigger<Completed<MoveAction>>,
    player: Single<(&mut LinearVelocity, &mut Sprite), (With<Player>, Without<BlockControls>)>,
    mut blasters: Single<&mut Visibility, With<PlayerBlasters>>,
) {
    let (mut velocity, mut sprite) = player.into_inner();

    velocity.0 = Vec2::default();
    let tl = Vec2::new(1., 4.) * 8.;
    let br = Vec2::new(2., 5.) * 8.;
    sprite.rect = Some(Rect::from_corners(tl, br));

    **blasters = Visibility::Hidden;
}

// TODO: this does not work? we don't brush on anything anyways

/// Brushing along edges rotates the player.
///
/// Transform and all physics things are synced in [`PhysicsSet::Sync`].
fn zero_rotation(mut player: Single<&mut Transform, With<Player>>) {
    player.rotation = Quat::default();
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

#[derive(InputContext)]
struct AliveContext;

fn handle_death(player: Single<Entity, (With<Player>, With<Dead>)>, mut commands: Commands) {
    info!("player died");
    commands.entity(*player).despawn();
}

fn handle_pickups(
    q: Single<(Entity, &mut WeaponEntity, &mut BulletRate), With<Player>>,
    mut events: EventReader<PickupEvent>,
    mut commands: Commands,
) {
    let (player, mut weapon_entity, mut rate) = q.into_inner();
    for event in events.read() {
        match event {
            PickupEvent::Weapon(Weapon::Bullet) => {
                commands.entity(weapon_entity.0).despawn();

                let emitter = commands
                    .spawn((DualEmitter::enemy(3.), Polarity::North))
                    .id();
                weapon_entity.0 = emitter;
                commands.entity(player).add_child(emitter);
            }
            PickupEvent::Weapon(Weapon::Missile) => {
                commands.entity(weapon_entity.0).despawn();

                let emitter = commands
                    .spawn((HomingEmitter::<EnemyType>::enemy(), Polarity::North))
                    .id();
                weapon_entity.0 = emitter;
                commands.entity(player).add_child(emitter);
            }
            PickupEvent::Upgrade(Upgrade::Speed(s)) => rate.0 += *s,
            //PickupEvent::Material => {}
            e => info!("handle: {e:?}"),
        }
    }
}

fn damage_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<BulletCollisionEvent>,
) {
    for event in reader.read() {
        if event.source == BulletSource::Enemy {
            commands.spawn((
                SamplePlayer::new(server.load("audio/sfx/laser.wav")),
                PlaybackSettings::ONCE,
                sample_effects![VolumeNode {
                    volume: Volume::Linear(0.2),
                }],
            ));
        }
    }
}
