use crate::pickups::{Pickup, PickupEvent};
use crate::player::{BlockControls, Materials, Player};
use crate::{GameState, assets, pickups};
use avian2d::prelude::{LinearVelocity, Physics, PhysicsTime};
use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_optix::shake::Shake;
use bevy_sequence::combinators::delay::AfterSystem;
use bevy_tween::bevy_time_runner::TimeRunner;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                enter_game_state.run_if(in_state(GameState::Game)),
                (update_selection, exit_game_state).run_if(in_state(GameState::Selection)),
            ),
        )
        .add_systems(OnEnter(GameState::Selection), enter_selection)
        .add_systems(OnExit(GameState::Selection), exit_selection);
    }
}

const MIN_MATERIALS: usize = 20;

fn enter_game_state(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Single<&mut Materials, With<Player>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        commands.set_state(GameState::Selection);
    }

    if player.get() >= MIN_MATERIALS {
        player.sub(MIN_MATERIALS);
        commands.set_state(GameState::Selection);
    }
}

fn exit_game_state(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    selection: Single<&Selection>,
    upgrades: Query<(&Pickup, &Upgrade)>,
    mut writer: EventWriter<PickupEvent>,
) {
    if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Space) {
        let selection = selection.0;
        let (pickup, _) = upgrades
            .iter()
            .find(|(_, upgrade)| upgrade.0 == selection)
            .unwrap();
        writer.write(pickup.into());
        commands.set_state(GameState::Game);
    }
}

#[derive(Component)]
struct SelectionEntity;

#[derive(Component)]
struct Upgrade(usize);

#[derive(Component)]
struct Frame(usize);

fn enter_selection(
    mut commands: Commands,
    mut time: ResMut<Time<Physics>>,
    server: Res<AssetServer>,
    mut tweens: Query<&mut TimeRunner>,
    mut shakes: Query<&mut Shake>,
    mut after_systems: Query<&mut AfterSystem>,
    player: Single<Entity, With<Player>>,
) {
    commands.entity(*player).insert(BlockControls);
    time.pause();
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

    commands.spawn((SelectionEntity, Selection(0)));
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
}

#[derive(Component)]
struct Selection(usize);

fn update_selection(
    input: Res<ButtonInput<KeyCode>>,
    mut selection: Single<&mut Selection>,
    mut frames: Query<(&mut Sprite, &Frame)>,
) {
    if input.just_pressed(KeyCode::KeyA) {
        if selection.0 == 0 {
            selection.0 = 2;
        } else {
            selection.0 -= 1;
        }
    }

    if input.just_pressed(KeyCode::KeyD) {
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
}

fn exit_selection(
    mut commands: Commands,
    mut time: ResMut<Time<Physics>>,
    mut tweens: Query<&mut TimeRunner>,
    mut shakes: Query<&mut Shake>,
    mut after_systems: Query<&mut AfterSystem>,
    entities: Query<Entity, With<SelectionEntity>>,
    player: Single<(Entity, &mut LinearVelocity), With<Player>>,
) {
    let (player, mut velocity) = player.into_inner();
    velocity.0 = Vec2::ZERO;
    commands.entity(player).remove::<BlockControls>();
    time.unpause();
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
