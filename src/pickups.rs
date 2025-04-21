use crate::assets;
use bevy::prelude::*;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

#[derive(Clone, Copy, Component)]
#[require(Transform)]
enum Upgrade {
    Speed(f32),
    Juice(f32),
}

impl Upgrade {
    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        match self {
            Self::Speed(_) => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(2, 1)),
            Self::Juice(_) => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(3, 1)),
        }
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    let upgrade = Upgrade::Speed(0.5);
    commands.spawn((
        upgrade,
        upgrade.sprite(&server),
        Transform::from_xyz(0., 30., 0.),
    ));

    let upgrade = Upgrade::Juice(0.5);
    commands.spawn((
        upgrade,
        upgrade.sprite(&server),
        Transform::from_xyz(0., -30., 0.),
    ));
}

#[derive(Component)]
enum Weapon {
    Bullet,
    Laser,
    Missile,
}

impl Weapon {
    pub fn sprite(&self, server: &AssetServer) -> Sprite {
        todo!("make sprites");
        //match self {
        //    Self::Bullet => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(2, 1)),
        //    Self::Laser => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(3, 1)),
        //    Self::Missile => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(3, 1)),
        //}
    }
}
