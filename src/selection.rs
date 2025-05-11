use crate::input::{self, MenuContext};
use crate::pickups::{Pickup, PickupEvent, ScrollingPickup};
use crate::player::{BlockControls, Materials, Player};
use crate::{GameState, assets, pickups};
use avian2d::prelude::{LinearVelocity, Physics, PhysicsTime};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_enhanced_input::events::Fired;
use bevy_enhanced_input::prelude::{ActionState, Actions};
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_optix::shake::Shake;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::AfterSystem;
use bevy_tween::bevy_time_runner::TimeRunner;

const INITIAL_MIN_MATERIALS: usize = 1;
const DISP_MSG: &str = "Remaining to \nUpgrade: ";

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MinMaterials(INITIAL_MIN_MATERIALS))
            .add_observer(exit_selection)
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(
                OnEnter(GameState::StartGame),
                (spawn_display, |mut materials: ResMut<MinMaterials>| {
                    materials.0 = INITIAL_MIN_MATERIALS
                }),
            )
            .add_systems(
                Update,
                (
                    (enter_selection, display_remaining).run_if(in_state(GameState::Game)),
                    update_selection.run_if(in_state(GameState::Selection)),
                ),
            )
            .add_systems(OnEnter(GameState::Selection), init_selection)
            .add_systems(OnExit(GameState::Selection), deinit_selection);

        #[cfg(debug_assertions)]
        app.add_systems(Update, selection_test);
    }
}

/// The minimum resources required for the next upgrade.
#[derive(Resource)]
pub struct MinMaterials(pub usize);

fn restart(mut commands: Commands, display: Single<Entity, With<Display>>) {
    commands.entity(*display).despawn();
}

fn selection_test(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyM) {
        commands.set_state(GameState::Selection);
    }
}

fn enter_selection(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut player: Single<(&mut Materials, &Transform), With<Player>>,
    mut mats: ResMut<MinMaterials>,
) {
    let (mut materials, pp) = player.into_inner();

    if materials.get() >= mats.0 {
        materials.sub(mats.0);
        mats.0 *= 2;
        //commands.set_state(GameState::Selection);

        let mut t = *pp;
        t.translation.y += 50.;
        commands.spawn((t, ScrollingPickup::new()));

        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/chimes.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.2),
                ..Default::default()
            },
        ));
    }
}

#[derive(Component)]
struct Display;

fn spawn_display(mut commands: Commands, server: Res<AssetServer>, mats: Res<MinMaterials>) {
    commands.spawn((
        Display,
        HIGH_RES_LAYER,
        Text2d(format!("{}{}", DISP_MSG, mats.0)),
        TextFont {
            font_size: 20.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_xyz(
            -crate::WIDTH / 2. * crate::RESOLUTION_SCALE,
            -crate::HEIGHT / 2. * crate::RESOLUTION_SCALE,
            500.,
        ),
        Anchor::BottomLeft,
    ));
}

fn display_remaining(
    player: Single<&Materials, (With<Player>, Changed<Materials>)>,
    mut display: Single<&mut Text2d, With<Display>>,
    mats: Res<MinMaterials>,
) {
    display.0 = format!("{}{}", DISP_MSG, mats.0.saturating_sub(player.get()));
}

fn exit_selection(
    _: Trigger<Fired<input::Interact>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    selection: Single<&Selection>,
    upgrades: Query<(&Pickup, &Upgrade)>,
    mut writer: EventWriter<PickupEvent>,
) {
    let selection = selection.0;
    let (pickup, _) = upgrades
        .iter()
        .find(|(_, upgrade)| upgrade.0 == selection)
        .unwrap();
    writer.write(pickup.into());
    commands.set_state(GameState::Game);

    commands.spawn((
        SamplePlayer::new(server.load("audio/sfx/chimes.wav")),
        PlaybackSettings {
            volume: Volume::Linear(0.2),
            ..Default::default()
        },
    ));
}

#[derive(Component)]
struct SelectionEntity;

#[derive(Component)]
struct Upgrade(usize);

#[derive(Component)]
struct Frame(usize);

#[derive(Component)]
struct InfoText;

fn init_selection(
    mut commands: Commands,
    mut physics_time: ResMut<Time<Physics>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    server: Res<AssetServer>,
    mut tweens: Query<&mut TimeRunner>,
    mut shakes: Query<&mut Shake>,
    mut after_systems: Query<&mut AfterSystem>,
    player: Single<Entity, With<Player>>,
) {
    commands.entity(*player).insert(BlockControls);
    physics_time.pause();
    virtual_time.pause();
    for mut runner in tweens.iter_mut() {
        runner.set_paused(true);
    }
    for mut shake in shakes.iter_mut() {
        shake.pause();
    }
    for mut after in after_systems.iter_mut() {
        after.pause();
    }

    commands.spawn((
        SelectionEntity,
        HIGH_RES_LAYER,
        Text2d("Choose Upgrade".into()),
        TextFont {
            font_size: 28.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_xyz(0., 80., 500.),
    ));
    commands.spawn((
        SelectionEntity,
        Sprite {
            rect: Some(Rect::from_center_size(
                Vec2::ZERO,
                Vec2::new(crate::WIDTH, crate::HEIGHT),
            )),
            color: Color::linear_rgba(0., 0., 0., 0.9),
            ..Default::default()
        },
        Transform::from_xyz(0., 0., 499.),
    ));

    commands.spawn((SelectionEntity, Selection(1)));
    let pickups = pickups::unique_pickups(3);
    info!("unique pickups: {pickups:?}");
    let positions = [
        Vec3::new(-30., 0., 500.),
        Vec3::new(0., 0., 500.),
        Vec3::new(30., 0., 500.),
    ];
    for (i, (pickup, position)) in pickups.iter().zip(positions.iter()).enumerate() {
        commands.spawn((
            SelectionEntity,
            Frame(i),
            Transform::from_translation(*position),
            assets::sprite_rect16(&server, assets::UI_PATH, UVec2::new(1, 4)),
        ));
        match *pickup {
            Pickup::Upgrade(upgrade) => {
                commands.spawn((
                    SelectionEntity,
                    Upgrade(i),
                    *pickup,
                    upgrade,
                    Transform::from_translation(*position),
                ));
            }
            Pickup::Weapon(weapon) => {
                commands.spawn((
                    SelectionEntity,
                    Upgrade(i),
                    *pickup,
                    weapon,
                    Transform::from_translation(*position),
                ));
            }
        }
    }

    commands.spawn((
        SelectionEntity,
        InfoText,
        HIGH_RES_LAYER,
        Text2d::default(),
        TextFont {
            font_size: 20.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_xyz(0., -crate::HEIGHT / 5. * crate::RESOLUTION_SCALE, 500.),
        Anchor::TopCenter,
    ));

    //commands.spawn((
    //    SelectionEntity,
    //    HIGH_RES_LAYER,
    //    Text2d("Press C to Select".into()),
    //    TextFont {
    //        font_size: 20.,
    //        font: server.load("fonts/joystix.otf"),
    //        ..Default::default()
    //    },
    //    Transform::from_xyz(
    //        0.,
    //        crate::HEIGHT / 2. * crate::RESOLUTION_SCALE - 120.,
    //        500.,
    //    ),
    //));
}

#[derive(Component)]
struct Selection(usize);

fn update_selection(
    input: Single<&Actions<MenuContext>>,
    mut selection: Single<&mut Selection>,
    mut frames: Query<(&mut Sprite, &Frame)>,
    options: Query<(&Upgrade, &Pickup)>,
    mut info: Single<&mut Text2d, With<InfoText>>,
) {
    if input.action::<input::Left>().state() == ActionState::Fired {
        if selection.0 == 0 {
            selection.0 = 2;
        } else {
            selection.0 -= 1;
        }
    }

    if input.action::<input::Right>().state() == ActionState::Fired {
        if selection.0 == 2 {
            selection.0 = 0;
        } else {
            selection.0 += 1;
        }
    }

    for (mut sprite, frame) in frames.iter_mut() {
        let select_rect = Some(assets::rect16(UVec2::new(0, 4)));
        if frame.0 == selection.0 {
            if sprite.rect != select_rect {
                sprite.rect = select_rect;
            }
        } else {
            if sprite.rect == select_rect {
                sprite.rect = Some(assets::rect16(UVec2::new(1, 4)));
            }
        }
    }

    if selection.is_changed() {
        let (_, pickup) = options.iter().find(|(u, _)| u.0 == selection.0).unwrap();
        match pickup {
            Pickup::Weapon(weapon) => match weapon {
                pickups::Weapon::Bullet => {
                    info.0 = "Dual machine guns".into();
                }
                pickups::Weapon::Missile => {
                    info.0 = "Homing missiles".into();
                }
                pickups::Weapon::Laser => {
                    info.0 = "BIG LASER".into();
                }
            },
            Pickup::Upgrade(upgrade) => match upgrade {
                pickups::Upgrade::Speed(s) => {
                    info.0 = format!("Increase shooting\nspeed by {:.2}%", s);
                }
                pickups::Upgrade::Juice(j) => {
                    info.0 = format!("Increase damage \nby {:.2}%", j);
                }
            },
        }
    }
}

fn deinit_selection(
    mut commands: Commands,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut physics_time: ResMut<Time<Physics>>,
    mut tweens: Query<&mut TimeRunner>,
    mut shakes: Query<&mut Shake>,
    mut after_systems: Query<&mut AfterSystem>,
    entities: Query<Entity, With<SelectionEntity>>,
    player: Single<(Entity, &mut LinearVelocity), With<Player>>,
) {
    let (player, mut velocity) = player.into_inner();
    velocity.0 = Vec2::ZERO;
    commands.entity(player).remove::<BlockControls>();
    virtual_time.unpause();
    physics_time.unpause();
    for mut runner in tweens.iter_mut() {
        runner.set_paused(false);
    }
    for mut shake in shakes.iter_mut() {
        shake.unpause();
    }
    for mut after in after_systems.iter_mut() {
        after.unpause();
    }

    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}
