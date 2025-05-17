use crate::animation::{AnimationAppExt, AnimationController, AnimationIndices, AnimationSprite};
use crate::assets::CHARACTERS_PATH;
use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_layout(
            CHARACTERS_PATH,
            TextureAtlasLayout::from_grid(UVec2::splat(8), 5, 11, None, None),
        )
        .add_systems(Update, spawn_characters);
    }
}

#[derive(Component)]
#[require(Transform)]
pub enum Character {
    Noise,
}

fn spawn_characters(mut commands: Commands, q: Query<(Entity, &Character), Without<Sprite>>) {
    for (entity, character) in q.iter() {
        let range = match character {
            Character::Noise => 50..=54,
        };
        commands
            .entity(entity)
            .insert(AnimationSprite::repeating(CHARACTERS_PATH, 0.5, range));
    }
}
