use crate::GameState;
use crate::enemy::EnemyDeathEvent;
use crate::pickups::{Material, PickupEvent};
use bevy::prelude::*;
use bevy::time::Stopwatch;

pub struct StatPlugin;

impl Plugin for StatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Stats::default())
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(Update, track_stats);
    }
}

fn restart(mut stats: ResMut<Stats>) {
    *stats = Stats::default();
}

#[derive(Default, Resource)]
pub struct Stats {
    pub time: GameTime,
    pub kills: usize,
    pub materials: usize,
}

pub struct GameTime {
    timer: Stopwatch,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            timer: Stopwatch::new(),
        }
    }
}

impl GameTime {
    pub fn elapsed_secs(&self) -> f32 {
        self.timer.elapsed_secs()
    }

    pub fn tick(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }
}

fn track_stats(
    mut stats: ResMut<Stats>,
    mut kills: EventReader<EnemyDeathEvent>,
    mut materials: EventReader<PickupEvent>,
    time: Res<Time>,
) {
    stats.time.tick(&time);
    stats.kills += kills.read().count();
    stats.materials += materials
        .read()
        .filter(
            |pickup| matches!(pickup, PickupEvent::Material(mat) if matches!(mat, Material::Parts)),
        )
        .count();
}
