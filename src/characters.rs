use crate::animation::{AnimationController, AnimationIndices};
use crate::assets::CHARACTERS_PATH;
use crate::atlas_layout;
use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_character_layout)
            .add_systems(Update, spawn_characters);
    }
}

atlas_layout!(CharacterLayout, init_character_layout, 8, 5, 11);

#[derive(Component)]
#[require(Transform)]
pub enum Character {
    Noise,
}

fn spawn_characters(
    mut commands: Commands,
    server: Res<AssetServer>,
    layout: Res<CharacterLayout>,
    q: Query<(Entity, &Character), Without<Sprite>>,
) {
    for (entity, character) in q.iter() {
        let atlas = TextureAtlas {
            layout: layout.0.clone(),
            index: 0,
        };

        let range = match character {
            Character::Noise => 50..=54,
        };
        commands.entity(entity).insert((
            Sprite::from_atlas_image(server.load(CHARACTERS_PATH), atlas),
            AnimationController::from_seconds(AnimationIndices::repeating(range), 0.5),
        ));
    }
}
