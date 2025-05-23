use crate::color::HexColor;
use crate::enemy::EnemyDeathEvent;
use crate::text::flash_text;
use crate::{GameState, RESOLUTION_SCALE};
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use std::usize;

pub const COLOR: HexColor = HexColor(0xfff540);
pub const POINT_TEXT_Z: f32 = 500.;

pub struct PointPlugin;

impl Plugin for PointPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointEvent>()
            .insert_resource(Points(0))
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(PostUpdate, (score_enemy_death, point_effects).chain());
    }
}

fn restart(mut commands: Commands) {
    commands.insert_resource(Points(0));
}

#[derive(Resource)]
pub struct Points(usize);

impl Points {
    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Event)]
pub struct PointEvent {
    pub points: usize,
    pub position: Vec2,
}

fn score_enemy_death(
    mut reader: EventReader<EnemyDeathEvent>,
    mut writer: EventWriter<PointEvent>,
) {
    for event in reader.read() {
        writer.write(PointEvent {
            points: 20,
            position: event.position,
        });
    }
}

fn point_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<PointEvent>,
    mut points: ResMut<Points>,
) {
    if !reader.is_empty() {
        //commands.spawn((
        //    SamplePlayer::new(server.load("audio/sfx/bfxr/Random50.wav")),
        //    PlaybackSettings {
        //        volume: Volume::Linear(0.35),
        //        ..PlaybackSettings::ONCE
        //    },
        //));
    }

    for event in reader.read() {
        points.0 += event.points;
        flash_text(
            &mut commands,
            &server,
            format!("+{}", event.points),
            20.,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            COLOR,
        );
    }
}
