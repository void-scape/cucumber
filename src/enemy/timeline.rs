use super::formation::*;
use crate::pickups::Weapon;
use crate::player::Player;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::time::Duration;

pub const LARGEST_SPRITE_SIZE: f32 = 16.;
pub const ENEMY_Z: f32 = 0.;

#[cfg(not(debug_assertions))]
const START_DELAY: f32 = 1.5;
#[cfg(debug_assertions)]
const START_DELAY: f32 = 0.;

pub fn start_waves(mut commands: Commands) {
    if crate::SKIP_WAVES {
        commands.insert_resource(WaveTimeline::new([(boss(), 0.)]));
    } else {
        commands.insert_resource(WaveTimeline::new_delayed(
            START_DELAY,
            [
                (swarm_right(), 0.),
                (swarm_left(), 2.),
                (crisscross().with(option(Weapon::Bullet)), 4.),
                (double_buck_shot(), 8.),
                (double_wall(), 2.),
                (double_crisscross(), 2.),
                (swarm_right(), 0.),
                (swarm_left(), 2.),
                (quad_mine_thrower(), 4.),
                (quad_mine_thrower().with(bomb), 6.),
                (double_crisscross(), 2.),
                (orb_slinger(), 6.),
                (laser_maze(), 2.),
                (double_wall(), 6.),
                (crisscross(), 2.),
                (double_orb_slinger().with(option(Weapon::Missile)), 2.),
                (double_wall(), 2.),
                (laser_ladder(), 4.),
                (swarm_right(), 0.),
                (swarm_left(), 4.),
                (swarm_right(), 0.),
                (swarm_left(), 3.8),
                (triple_wall(), 5.),
                (double_buck_shot(), 3.),
                (quad_mine_thrower(), 4.),
                (quad_mine_thrower().with(bomb), 18.),
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
    pub fn new(seq: impl IntoIterator<Item = (Formation, f32)>) -> Self {
        Self::new_delayed(0., seq)
    }

    pub fn new_delayed(delay: f32, seq: impl IntoIterator<Item = (Formation, f32)>) -> Self {
        Self {
            seq: seq.into_iter().collect(),
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

    pub fn next(&mut self) -> Option<&mut Formation> {
        if self.timer.just_finished() {
            match self.seq.get_mut(self.index) {
                Some((formation, duration)) => {
                    self.timer.set_duration(Duration::from_secs_f32(*duration));
                    self.index += 1;
                    Some(formation)
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
    server: Res<AssetServer>,
    controller: Option<ResMut<WaveTimeline>>,
    time: Res<Time>,
) {
    let Some(mut controller) = controller else {
        return;
    };

    if controller.finished() {
        return;
    }

    //const PADDING: f32 = LARGEST_SPRITE_SIZE;
    //const FORMATION_EASE_DUR: f32 = 2.;

    controller.tick(&time);
    if let Some(formation) = controller.next() {
        let start_y = crate::HEIGHT / 2. + LARGEST_SPRITE_SIZE / 2.;
        let start = Vec3::new(0., start_y, ENEMY_Z);

        //commands.entity(root).animation().insert(tween(
        //    Duration::from_secs_f32(FORMATION_EASE_DUR),
        //    EaseKind::SineOut,
        //    root.into_target().with(translation(start, end)),
        //));

        let mut commands = commands.spawn((
            FormationEntity(formation.velocity),
            Transform::from_translation(start),
        ));
        (formation.spawn)(&mut commands, &server);
        for modifier in formation.modifiers.iter_mut() {
            modifier(&mut commands);
        }
    }
}
