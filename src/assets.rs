#![allow(unused)]

use crate::HEIGHT;
use crate::animation::AnimationAppExt;
use crate::atlas_layout;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

pub const BACKGROUNDS_PATH: &'static str = "shooters/SpaceShooterAssetPack_BackGrounds.png";
pub const CHARACTERS_PATH: &'static str = "shooters/SpaceShooterAssetPack_Characters.png";
pub const UI_PATH: &'static str = "shooters/SpaceShooterAssetPack_IU.png";
pub const MISC_PATH: &'static str = "shooters/SpaceShooterAssetPack_Miscellaneous.png";
pub const PROJECTILES_PATH: &'static str =
    "shooters/SpaceShooterAssetPack_Projectiles_Grayscale.png";
pub const PROJECTILES_COLORED_PATH: &'static str = "shooters/SpaceShooterAssetPack_Projectiles.png";
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
        rect: Some(rect8(cell)),
        ..Default::default()
    }
}

fn rect8(cell: UVec2) -> Rect {
    Rect::from_corners(cell.as_vec2() * 8., (cell.as_vec2() + 1.) * 8.)
}

pub fn sprite_rect16(server: &AssetServer, path: &'static str, cell: UVec2) -> Sprite {
    Sprite {
        image: server.load(path),
        rect: Some(rect16(cell)),
        ..Default::default()
    }
}

pub fn rect16(cell: UVec2) -> Rect {
    Rect::from_corners(cell.as_vec2() * 16., (cell.as_vec2() + 1.) * 16.)
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, insert_sprite).register_layout(
            MISC_PATH,
            TextureAtlasLayout::from_grid(UVec2::splat(8), 13, 8, None, None),
        );

        #[cfg(debug_assertions)]
        app.add_systems(Update, debug_assets);
    }
}

#[derive(Component)]
pub struct AutoSprite {
    pub path: &'static str,
    pub cell: UVec2,
    pub size: SpriteSize,
}

impl AutoSprite {
    pub fn new(path: &'static str, cell: UVec2, size: SpriteSize) -> Self {
        Self { path, cell, size }
    }

    pub fn new8(path: &'static str, cell: UVec2) -> Self {
        Self::new(path, cell, SpriteSize::Size8)
    }

    pub fn new16(path: &'static str, cell: UVec2) -> Self {
        Self::new(path, cell, SpriteSize::Size16)
    }
}

pub enum SpriteSize {
    Size8,
    Size16,
}

fn insert_sprite(
    mut commands: Commands,
    server: Res<AssetServer>,
    auto_sprites: Query<(Entity, &AutoSprite)>,
) {
    for (entity, sprite) in auto_sprites.iter() {
        match sprite.size {
            SpriteSize::Size8 => {
                commands
                    .entity(entity)
                    .insert(sprite_rect8(&server, sprite.path, sprite.cell))
                    .remove::<AutoSprite>();
            }
            SpriteSize::Size16 => {
                commands
                    .entity(entity)
                    .insert(sprite_rect16(&server, sprite.path, sprite.cell))
                    .remove::<AutoSprite>();
            }
        }
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
                    commands.entity(asset).despawn();
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

#[macro_export]
macro_rules! atlas_layout {
    ($name:ident, $func:ident, $cellsize:expr, $width:expr, $height:expr) => {
        #[derive(Resource)]
        pub struct $name(pub Handle<TextureAtlasLayout>);

        fn $func(mut commands: Commands, mut layouts: ResMut<Assets<TextureAtlasLayout>>) {
            let layout = layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat($cellsize),
                $width,
                $height,
                None,
                None,
            ));
            commands.insert_resource($name(layout));
        }
    };
}
