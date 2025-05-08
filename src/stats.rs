use crate::GameState;
use crate::enemy::EnemyDeathEvent;
use crate::miniboss::BossDeathEvent;
use crate::pickups::PickupEvent;
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
    should_tick: bool,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            timer: Stopwatch::new(),
            should_tick: true,
        }
    }
}

impl GameTime {
    pub fn elapsed_secs(&self) -> f32 {
        self.timer.elapsed_secs()
    }

    pub fn tick(&mut self, time: &Time) {
        if self.should_tick {
            self.timer.tick(time.delta());
        }
    }
}

fn track_stats(
    mut stats: ResMut<Stats>,
    mut kills: EventReader<EnemyDeathEvent>,
    mut materials: EventReader<PickupEvent>,
    mut boss_death: EventReader<BossDeathEvent>,
    time: Res<Time>,
) {
    if boss_death.read().count() > 0 {
        stats.time.should_tick = false;
    }
    stats.time.tick(&time);
    stats.kills += kills.read().count();
    stats.materials += materials
        .read()
        .filter(|pickup| matches!(pickup, PickupEvent::Material))
        .count();
}
