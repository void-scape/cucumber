use crate::enemy::formation::Formation;
use crate::{GameState, boss};
use bevy::prelude::*;
use bevy_seedling::prelude::*;

const MUSIC_VOLUME: f32 = 0.4;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(OnEnter(GameState::StartGame), startup)
            .add_systems(Update, update_layers);
    }
}

fn restart(mut commands: Commands, players: Query<Entity, With<SamplePlayer>>) {
    for entity in players.iter() {
        commands.entity(entity).despawn();
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        WaveLayer,
        SamplePlayer::new(server.load("audio/music/robbery_base.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::Linear(MUSIC_VOLUME),
        }],
    ));

    commands.spawn((
        WaveLayer,
        WaveCombatLayer,
        SamplePlayer::new(server.load("audio/music/robbery_lead.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::SILENT,
        }],
    ));
    commands.spawn((
        WaveLayer,
        WaveCombatLayer,
        SamplePlayer::new(server.load("audio/music/robbery_drums.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::SILENT,
        }],
    ));

    commands.spawn((
        BossLayer,
        SamplePlayer::new(server.load("audio/music/something_imminent.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::SILENT
        }],
    ));

    commands.spawn((
        BossLayer,
        BossBLayer,
        SamplePlayer::new(server.load("audio/music/something_imminent_arp.wav")),
        PlaybackSettings::LOOP,
        sample_effects![VolumeNode {
            volume: Volume::SILENT,
        }],
    ));
}

#[derive(Component)]
struct WaveLayer;

#[derive(Component)]
struct WaveCombatLayer;

#[derive(Component)]
struct BossLayer;

#[derive(Component)]
struct BossBLayer;

fn update_layers(
    wave_base: Single<&SampleEffects, (With<WaveLayer>, Without<WaveCombatLayer>)>,
    wave_combat_layers: Query<&SampleEffects, With<WaveCombatLayer>>,
    all_wave: Query<&SampleEffects, With<WaveLayer>>,
    boss_base: Single<&SampleEffects, (With<BossLayer>, Without<BossBLayer>)>,
    boss_b_layer: Single<&SampleEffects, (With<BossLayer>, With<BossBLayer>)>,
    all_boss: Query<&SampleEffects, With<BossLayer>>,
    mut nodes: Query<&mut VolumeNode>,
    formations: Query<&Formation>,
    boss: Option<Single<&boss::gradius::Gradius>>,
    boss_b: Option<Single<&boss::gradius::PhaseB>>,
) -> Result {
    if boss.is_none() {
        nodes.get_effect_mut(*wave_base)?.volume = Volume::Linear(MUSIC_VOLUME);

        for effects in wave_combat_layers.iter() {
            let mut node = nodes.get_effect_mut(effects)?;

            if !formations.is_empty() {
                node.volume = Volume::Linear(MUSIC_VOLUME);
            } else {
                node.volume = Volume::SILENT;
            }
        }

        for effects in all_boss.iter() {
            let mut node = nodes.get_effect_mut(effects)?;
            node.volume = Volume::SILENT;
        }
    } else {
        nodes.get_effect_mut(*boss_base)?.volume = Volume::Linear(MUSIC_VOLUME);
        if boss_b.is_some() {
            nodes.get_effect_mut(*boss_b_layer)?.volume = Volume::Linear(MUSIC_VOLUME);
        } else {
            nodes.get_effect_mut(*boss_b_layer)?.volume = Volume::SILENT;
        }

        for effects in all_wave.iter() {
            let mut node = nodes.get_effect_mut(effects)?;
            node.volume = Volume::SILENT;
        }
    }

    Ok(())
}
