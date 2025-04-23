use crate::health::{Health, HealthSet};
use crate::pickups::PickupEvent;
use crate::player::{PLAYER_HEALTH, Player};
use crate::{GameState, RES_HEIGHT, RES_WIDTH, RESOLUTION_SCALE};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_pixel_gfx::pixel_perfect::HIGH_RES_LAYER;
use physics::Physics;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), (frame, health, init_upgrade_ui))
            .add_systems(Physics, update_health.after(HealthSet))
            .add_systems(Update, update_upgrades);
    }
}

fn init_upgrade_ui(mut commands: Commands) {
    commands.spawn(UpgradeUi(0));
}

fn frame(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
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
    if PLAYER_HEALTH == usize::MAX {
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
    for _ in 0..PLAYER_HEALTH {
        commands.spawn((
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
    player: Query<&Health, With<Player>>,
) {
    let Ok(health) = player.get_single() else {
        return;
    };

    let current_health = health.current();
    for (i, (mut sprite, _)) in q
        .iter_mut()
        .sort_unstable_by::<&Transform>(|a, b| b.translation.y.total_cmp(&a.translation.y))
        .enumerate()
    {
        sprite.texture_atlas.as_mut().unwrap().index = (i + 1 > current_health) as usize;
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
    mut ui: Query<&mut UpgradeUi>,
) {
    let Ok(mut upgrade_ui) = ui.get_single_mut() else {
        return;
    };

    for event in reader.read() {
        match event {
            PickupEvent::Upgrade(upgrade) => {
                let x = (RES_WIDTH / 2. - UPGRADE_OFFSET) * RESOLUTION_SCALE;
                let mut y = (RES_HEIGHT / 2. - UPGRADE_OFFSET) * RESOLUTION_SCALE;
                y -= (upgrade_ui.0 as f32) * (UPGRADE_SIZE + UPGRADE_OFFSET) * RESOLUTION_SCALE;
                let mut sprite = upgrade.sprite(&server);
                sprite.anchor = Anchor::TopRight;
                commands.spawn((
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
