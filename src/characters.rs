use crate::assets::CHARACTERS_PATH;
use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, animate_sprites);
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
        AnimationController {
            indices: AnimationIndices { first: 0, last: 4 },
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        },
    ));
}

#[derive(Component)]
struct AnimationController {
    indices: AnimationIndices,
    timer: Timer,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

fn animate_sprites(time: Res<Time>, mut query: Query<(&mut AnimationController, &mut Sprite)>) {
    for (mut controller, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        controller.timer.tick(time.delta());
        if controller.timer.just_finished() {
            atlas.index = if atlas.index == controller.indices.last {
                controller.indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}
