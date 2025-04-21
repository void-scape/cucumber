use crate::animation::{AnimationController, AnimationIndices, AnimationMode};
use crate::assets::CHARACTERS_PATH;
use crate::{HEIGHT, WIDTH};
use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(8),
        5,
        11,
        None,
        None,
    ));
    let atlas = TextureAtlas { layout, index: 0 };

    commands.spawn((
        Sprite::from_atlas_image(server.load(CHARACTERS_PATH), atlas),
        AnimationController::from_seconds(AnimationIndices::new(AnimationMode::Repeat, 0..5), 0.5),
        Transform::from_xyz(-WIDTH / 2. + 20., -HEIGHT / 2. + 20., 0.),
    ));
}
