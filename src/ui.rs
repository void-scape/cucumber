use crate::health::{Health, Shield};
use crate::pickups::PickupEvent;
use crate::player::{PLAYER_HEALTH, PLAYER_SHIELD, Player};
use crate::{GameState, RES_HEIGHT, RES_WIDTH, RESOLUTION_SCALE};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(
                OnEnter(GameState::StartGame),
                (frame, health, shield, init_upgrade_ui),
            )
            .add_systems(Update, (update_health, update_upgrades, update_shield));
    }
}

fn restart(mut commands: Commands, ui: Query<Entity, With<UI>>) {
    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct UI;

fn init_upgrade_ui(mut commands: Commands) {
    commands.spawn((UI, UpgradeUi(0)));
}

fn frame(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        UI,
        HIGH_RES_LAYER,
        Sprite::from_image(server.load("frame.png")),
        Transform::from_scale(Vec3::splat(RESOLUTION_SCALE)),
    ));
}

#[derive(Component)]
struct HeartUi;

const HEART_OFFSET: f32 = 2.;
const HEART_SIZE: f32 = 10.;

fn health(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if PLAYER_HEALTH == f32::MAX {
        return;
    }

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(10),
        2,
        1,
        None,
        None,
    ));
    let atlas = TextureAtlas { layout, index: 0 };

    let x = (-RES_WIDTH / 2. + HEART_OFFSET) * RESOLUTION_SCALE;
    let mut y = (RES_HEIGHT / 2. - HEART_OFFSET) * RESOLUTION_SCALE;
    for _ in 0..(PLAYER_HEALTH as usize) {
        commands.spawn((
            UI,
            HeartUi,
            Sprite {
                image: server.load("heart_ui.png"),
                texture_atlas: Some(atlas.clone()),
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            Transform::from_xyz(x, y, 0.).with_scale(Vec3::splat(RESOLUTION_SCALE)),
            HIGH_RES_LAYER,
        ));
        y -= (HEART_SIZE + HEART_OFFSET) * RESOLUTION_SCALE;
    }
}

fn update_health(
    mut q: Query<(&mut Sprite, &Transform), With<HeartUi>>,
    health: Single<&Health, With<Player>>,
) {
    let current_health = health.current();
    for (i, (mut sprite, _)) in q
        .iter_mut()
        .sort_unstable_by::<&Transform>(|a, b| b.translation.y.total_cmp(&a.translation.y))
        .enumerate()
    {
        sprite.texture_atlas.as_mut().unwrap().index = (i + 1 > current_health as usize) as usize;
    }
}

#[derive(Component)]
struct ShieldUi;

fn shield(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if PLAYER_HEALTH == f32::MAX {
        return;
    }

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        2,
        1,
        None,
        None,
    ));
    let atlas = TextureAtlas { layout, index: 0 };

    let x = (-RES_WIDTH / 2.) * RESOLUTION_SCALE;
    let mut y = (RES_HEIGHT / 2. - HEART_OFFSET) * RESOLUTION_SCALE
        - (HEART_SIZE + HEART_OFFSET) * RESOLUTION_SCALE * PLAYER_HEALTH;
    for _ in 0..(PLAYER_SHIELD as usize) {
        commands.spawn((
            UI,
            ShieldUi,
            Sprite {
                image: server.load("shield_ui.png"),
                texture_atlas: Some(atlas.clone()),
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            Transform::from_xyz(x, y, 0.).with_scale(Vec3::splat(RESOLUTION_SCALE)),
            HIGH_RES_LAYER,
        ));
        y -= (HEART_SIZE + HEART_OFFSET) * RESOLUTION_SCALE;
    }
}

fn update_shield(
    mut q: Query<(&mut Sprite, &Transform), With<ShieldUi>>,
    shield: Single<&Shield, With<Player>>,
) {
    let current_shield = shield.current();
    for (i, (mut sprite, _)) in q
        .iter_mut()
        .sort_unstable_by::<&Transform>(|a, b| b.translation.y.total_cmp(&a.translation.y))
        .enumerate()
    {
        sprite.texture_atlas.as_mut().unwrap().index = (i + 1 > current_shield as usize) as usize;
    }
}

#[derive(Component)]
struct UpgradeUi(usize);

const UPGRADE_OFFSET: f32 = 4.;
const UPGRADE_SIZE: f32 = 8.;

fn update_upgrades(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<PickupEvent>,
    mut upgrade_ui: Single<&mut UpgradeUi>,
) {
    for event in reader.read() {
        match event {
            PickupEvent::Upgrade(upgrade) => {
                let x = (RES_WIDTH / 2. - UPGRADE_OFFSET) * RESOLUTION_SCALE;
                let mut y = (RES_HEIGHT / 2. - UPGRADE_OFFSET) * RESOLUTION_SCALE;
                y -= (upgrade_ui.0 as f32) * (UPGRADE_SIZE + UPGRADE_OFFSET) * RESOLUTION_SCALE;
                let mut sprite = upgrade.sprite(&server);
                sprite.anchor = Anchor::TopRight;
                commands.spawn((
                    UI,
                    sprite,
                    Transform::from_xyz(x, y, 0.).with_scale(Vec3::splat(RESOLUTION_SCALE)),
                    HIGH_RES_LAYER,
                ));
                upgrade_ui.0 += 1;
            }
            _ => {}
        }
    }
}
