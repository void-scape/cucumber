use super::formation::*;
use crate::player::Player;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

#[cfg(not(debug_assertions))]
const START_DELAY: f32 = 1.5;
#[cfg(debug_assertions)]
const START_DELAY: f32 = 0.;

pub fn start_waves(mut commands: Commands) {
    if crate::SKIP_WAVES {
        commands.insert_resource(WaveTimeline::new(&[(boss(), 0.)]));
    } else {
        commands.insert_resource(WaveTimeline::new_delayed(
            START_DELAY,
            &[
                //(swarm(), 8.),
                (crisscross(), 2.),
                (double_buck_shot(), 4.),
                (swarm(), 6.),
                (double_buck_shot(), 8.),
                (quad_mine_thrower(), 4.),
                (swarm(), 4.),
                (quad_mine_thrower(), 8.),
                (double_crisscross(), 2.),
                (orb_slinger(), 8.),
                (crisscross(), 2.),
                (double_orb_slinger(), 10.),
                (swarm(), 16.),
                (boss(), 0.),
            ],
        ));
    }
}

#[derive(Resource)]
pub struct WaveTimeline {
    seq: Vec<(Formation, f32)>,
    timer: Timer,
    index: usize,
    finished: bool,
    skip: Option<Timer>,
}

impl WaveTimeline {
    pub fn new(seq: &[(Formation, f32)]) -> Self {
        Self::new_delayed(0., seq)
    }

    pub fn new_delayed(delay: f32, seq: &[(Formation, f32)]) -> Self {
        Self {
            seq: seq.to_vec(),
            timer: Timer::from_seconds(delay, TimerMode::Repeating),
            index: 0,
            finished: false,
            skip: None,
        }
    }

    pub fn skip(mut self, secs: f32) -> Self {
        self.skip = Some(Timer::from_seconds(secs, TimerMode::Once));
        self
    }

    pub fn is_skipping(&self) -> bool {
        self.skip.is_some()
    }

    pub fn tick(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }

    pub fn next(&mut self) -> Option<Formation> {
        if self.timer.just_finished() {
            match self.seq.get(self.index) {
                Some((formation, duration)) => {
                    self.timer.set_duration(Duration::from_secs_f32(*duration));
                    self.index += 1;
                    Some(formation.clone())
                }
                None => {
                    self.finished = true;
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn finished(&self) -> bool {
        self.finished
    }
}

#[cfg(debug_assertions)]
pub fn timeline_skip(
    mut commands: Commands,
    controller: Option<ResMut<WaveTimeline>>,
    mut time: ResMut<Time<Virtual>>,
    player: Single<Entity, With<Player>>,
) {
    let Some(mut controller) = controller else {
        return;
    };

    if controller.is_added() && controller.skip.is_some() {
        commands.entity(*player).insert(ColliderDisabled);
    }

    let Some(timer) = controller.skip.as_mut() else {
        return;
    };

    time.advance_by(Duration::from_millis(16));
    timer.tick(time.delta());
    if timer.finished() {
        controller.skip = None;
        commands.entity(*player).remove::<ColliderDisabled>();
    }
}

pub fn update_waves(
    mut commands: Commands,
    controller: Option<ResMut<WaveTimeline>>,
    time: Res<Time>,
) {
    let Some(mut controller) = controller else {
        return;
    };

    if controller.finished() {
        return;
    }

    controller.tick(&time);
    if let Some(formation) = controller.next() {
        commands.spawn(formation);
    }
}
