use crate::GameState;
use crate::enemy::Formation;
use bevy::prelude::*;
use bevy_seedling::prelude::*;

const MUSIC_VOLUME: f32 = 0.4;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(OnEnter(GameState::Game), startup)
        //    .add_systems(Update, update_layers);
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        SamplePlayer::new(server.load("audio/music/robbery_base.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::Linear(MUSIC_VOLUME),
        }],
    ));

    commands.spawn((
        CombatLayer,
        SamplePlayer::new(server.load("audio/music/robbery_lead.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::SILENT,
        }],
    ));
    commands.spawn((
        CombatLayer,
        SamplePlayer::new(server.load("audio/music/robbery_drums.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::SILENT,
        }],
    ));
}

#[derive(Component)]
struct CombatLayer;

fn update_layers(
    combat_layers: Query<&SampleEffects, With<CombatLayer>>,
    mut nodes: Query<&mut VolumeNode>,
    formations: Query<&Formation>,
) -> Result {
    for effects in combat_layers.iter() {
        let mut node = nodes.get_effect_mut(effects)?;

        if !formations.is_empty() {
            node.volume = Volume::Linear(MUSIC_VOLUME);
        } else {
            node.volume = Volume::SILENT;
        }
    }

    Ok(())
}
