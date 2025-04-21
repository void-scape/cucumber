use crate::enemy::Formation;
use bevy::prelude::*;
use bevy_seedling::prelude::*;

const MUSIC_VOLUME: f32 = 0.5;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, update_layers);
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    commands
        .spawn((
            SamplePlayer::new(server.load("audio/music/robbery_base.wav")),
            PlaybackSettings::LOOP,
        ))
        .effect(VolumeNode {
            volume: Volume::Linear(MUSIC_VOLUME),
        });

    commands
        .spawn((
            CombatLayer,
            SamplePlayer::new(server.load("audio/music/robbery_lead.wav")),
            PlaybackSettings::LOOP,
        ))
        .effect(VolumeNode {
            volume: Volume::SILENT,
        });
    commands
        .spawn((
            CombatLayer,
            SamplePlayer::new(server.load("audio/music/robbery_drums.wav")),
            PlaybackSettings::LOOP,
        ))
        .effect(VolumeNode {
            volume: Volume::SILENT,
        });
}

#[derive(Component)]
struct CombatLayer;

fn update_layers(
    mut combat_layers: Query<&mut VolumeNode, With<CombatLayer>>,
    formations: Query<&Formation>,
) {
    for mut node in combat_layers.iter_mut() {
        if !formations.is_empty() {
            node.volume = Volume::Linear(MUSIC_VOLUME);
        } else {
            node.volume = Volume::SILENT;
        }
    }
}
