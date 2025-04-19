use bevy::prelude::*;

pub struct InvadersSpritePlugin;

impl Plugin for InvadersSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        bevy_pixel_gfx::pixel_perfect::HIGH_RES_LAYER,
        Sprite::from_image(server.load("invaders_sprites.png")),
    ));
}
