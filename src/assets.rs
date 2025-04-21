#![allow(unused)]

use crate::HEIGHT;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

pub const BACKGROUNDS_PATH: &'static str = "shooters/SpaceShooterAssetPack_BackGrounds.png";
pub const CHARACTERS_PATH: &'static str = "shooters/SpaceShooterAssetPack_Characters.png";
pub const UI_PATH: &'static str = "shooters/SpaceShooterAssetPack_IU.png";
pub const MISC_PATH: &'static str = "shooters/SpaceShooterAssetPack_Miscellaneous.png";
pub const PROJECTILES_PATH: &'static str = "shooters/SpaceShooterAssetPack_Projectiles.png";
pub const SHIPS_PATH: &'static str = "shooters/SpaceShooterAssetPack_Ships.png";
pub const WEAPONS_PATH: &'static str = "shooters/SpaceShooterAssetPack_Weapons.png";

const ASSETS: &[&str] = &[
    //BACKGROUNDS_PATH,
    //CHARACTERS_PATH,
    //UI_PATH,
    //MISC_PATH,
    PROJECTILES_PATH,
    // SHIPS_PATH,
];

pub fn sprite_rect8(server: &AssetServer, path: &'static str, cell: UVec2) -> Sprite {
    Sprite {
        image: server.load(path),
        rect: Some(Rect::from_corners(
            cell.as_vec2() * 8.,
            (cell.as_vec2() + 1.) * 8.,
        )),
        ..Default::default()
    }
}

pub fn sprite_rect16(server: &AssetServer, path: &'static str, cell: UVec2) -> Sprite {
    Sprite {
        image: server.load(path),
        rect: Some(Rect::from_corners(
            cell.as_vec2() * 16.,
            (cell.as_vec2() + 1.) * 16.,
        )),
        ..Default::default()
    }
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_assets);
    }
}

#[derive(Component)]
struct DebugAsset;

fn debug_assets(
    mut commands: Commands,
    mut reader: EventReader<KeyboardInput>,
    server: Res<AssetServer>,
    mut spawned: Local<bool>,
    assets: Query<Entity, With<DebugAsset>>,
) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::KeyL {
            if *spawned {
                for asset in assets.iter() {
                    commands.entity(asset).despawn_recursive();
                }
            } else {
                let mut y = HEIGHT / 2.;
                for (i, path) in ASSETS.iter().enumerate() {
                    if i % 2 == 0 {
                        y -= 100.;
                    }

                    commands.spawn((
                        DebugAsset,
                        Transform::from_xyz(0., y, 999.),
                        Sprite::from_image(server.load(*path)),
                    ));
                }
            }

            *spawned = !*spawned;
        }
    }
}
