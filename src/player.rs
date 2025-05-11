use crate::{
    GameState, HEIGHT, Layer,
    animation::{AnimationController, AnimationIndices},
    assets::{self, MISC_PATH, MiscLayout},
    bullet::{
        BulletTimer, Polarity,
        emitter::{BulletModifiers, DualEmitter, HomingEmitter, LaserEmitter},
    },
    end,
    enemy::Enemy,
    health::{Dead, Health, Shield},
    pickups::{Material, PickupEvent, Upgrade, Weapon},
};
use avian2d::prelude::*;
use bevy::{
    ecs::{component::HookContext, system::RunSystemOnce, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_sequence::combinators::delay::run_after;
#[cfg(not(debug_assertions))]
use bevy_tween::{
    interpolate::translation,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use std::{cmp::Ordering, f32, time::Duration};

pub const PLAYER_HEALTH: f32 = 2.0;
pub const PLAYER_SHIELD: f32 = 4.0;
const PLAYER_EASE_DUR: f32 = 1.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(OnEnter(GameState::StartGame), |mut commands: Commands| {
                let starting_weapon = commands.spawn(Player::bullet_emitter()).id();
                let player = commands
                    .spawn((
                        Player,
                        WeaponEntity(starting_weapon),
                        Transform::from_xyz(0., -HEIGHT / 6., 0.),
                    ))
                    .add_child(starting_weapon)
                    .id();
                //commands.spawn((Miner, MinerLeader(player)));

                let dur = Duration::from_secs_f32(PLAYER_EASE_DUR);
                run_after(
                    dur,
                    move |mut commands: Commands| {
                        commands.entity(player).remove::<BlockControls>();
                    },
                    &mut commands,
                );

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
            })
            .add_systems(
                Update,
                (
                    handle_pickups,
                    handle_death,
                    zero_rotation,
                    update_player_sprites,
                ),
            )
            .add_input_context::<AliveContext>()
            .add_observer(apply_movement)
            .add_observer(stop_movement);
    }
}

fn restart(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands.entity(*player).despawn();
}

#[derive(Component)]
#[require(
    Transform,
    LinearVelocity,
    Shield::full(PLAYER_SHIELD),
    Health::full(PLAYER_HEALTH),
    RigidBody::Dynamic,
    Collider::rectangle(6., 6.),
    CollidingEntities,
    CollisionLayers::new(Layer::Player, [Layer::Bounds, Layer::Bullet, Layer::Collectable]),
    BulletModifiers,
    Materials,
)]
#[component(on_add = Self::on_add)]
pub struct Player;

impl Player {
    pub fn bullet_emitter() -> impl Bundle {
        (
            DualEmitter::enemy(3.),
            BulletModifiers {
                damage: 0.5,
                rate: 2.,
                speed: 1.5,
            },
            Polarity::North,
        )
    }

    pub fn missile_emitter() -> impl Bundle {
        (HomingEmitter::<Enemy>::enemy(), Polarity::North)
    }

    pub fn laser_emitter() -> impl Bundle {
        (
            LaserEmitter::enemy(),
            BulletModifiers {
                damage: 0.2,
                ..Default::default()
            },
            Polarity::North,
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
pub struct BlockControls;

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    player: Single<(&mut LinearVelocity, Option<&BlockControls>), With<Player>>,
) {
    let (mut velocity, blocked) = player.into_inner();

    if blocked.is_none() {
        velocity.0 = trigger.value.normalize_or_zero() * 60.;
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

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

#[derive(InputContext)]
struct AliveContext;

fn handle_death(mut commands: Commands, player: Single<Entity, (With<Player>, With<Dead>)>) {
    commands.entity(*player).despawn();
    commands.queue(|world: &mut World| world.run_system_once(end::show_loose_screen));
}

fn handle_pickups(
    mut commands: Commands,
    q: Single<
        (
            Entity,
            &mut WeaponEntity,
            &mut BulletModifiers,
            &mut Materials,
            &mut Shield,
        ),
        With<Player>,
    >,
    mut events: EventReader<PickupEvent>,
) {
    let (player, mut weapon_entity, mut mods, mut materials, mut shield) = q.into_inner();
    for event in events.read() {
        match event {
            PickupEvent::Weapon(weapon) => {
                // commands.entity(weapon_entity.0).despawn();

                let emitter = match weapon {
                    Weapon::Bullet => commands.spawn(Player::bullet_emitter()).id(),
                    Weapon::Missile => commands.spawn(Player::missile_emitter()).id(),
                    Weapon::Laser => commands.spawn(Player::laser_emitter()).id(),
                };

                weapon_entity.0 = emitter;
                commands.entity(player).add_child(emitter);

                commands.run_system_cached(
                    |player: Single<&Children, With<Player>>,
                     mut children: Query<&mut Transform, With<BulletModifiers>>| {
                        let total = children.iter_many(player.iter()).count() as f32;

                        let padding = Vec3::new(4.0, 0.0, 0.0);
                        let start = padding * -0.5 * total;

                        let mut children = children.iter_many_mut(player.iter());
                        let mut i = 0;

                        while let Some(mut transform) = children.fetch_next() {
                            transform.translation = start + padding * i as f32;

                            i += 1;
                        }
                    },
                );
            }
            PickupEvent::Upgrade(Upgrade::Speed(s)) => mods.rate += *s,
            PickupEvent::Upgrade(Upgrade::Juice(j)) => mods.damage += *j,
            PickupEvent::Material(mat) => match mat {
                Material::Parts => materials.0 += 1,
                Material::Shield => shield.heal(1. / 10.),
            },
        }
    }
}
