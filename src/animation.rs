use avian2d::prelude::Physics;
use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (init_atlas_index, animate_sprites));
    }
}

#[derive(Component)]
pub struct AnimationController {
    indices: AnimationIndices,
    timer: Timer,
}

impl AnimationController {
    pub fn new(indices: AnimationIndices, timer: Timer) -> Self {
        Self { indices, timer }
    }

    pub fn from_seconds(indices: AnimationIndices, secs: f32) -> Self {
        Self::new(indices, Timer::from_seconds(secs, TimerMode::Repeating))
    }
}

#[derive(Component)]
pub struct AnimationIndices {
    mode: AnimationMode,
    index: usize,
    seq: Vec<usize>,
}

impl AnimationIndices {
    #[track_caller]
    pub fn new(mode: AnimationMode, seq: impl IntoIterator<Item = usize>) -> Self {
        let seq = seq.into_iter().collect::<Vec<_>>();
        assert!(
            !seq.is_empty(),
            "tried to insert empty sequence into `AnimationIndices`"
        );

        Self {
            mode,
            index: 0,
            seq,
        }
    }

    pub fn repeating(seq: impl IntoIterator<Item = usize>) -> Self {
        Self::new(AnimationMode::Repeat, seq)
    }

    pub fn once_despawn(seq: impl IntoIterator<Item = usize>) -> Self {
        Self::new(AnimationMode::Despawn, seq)
    }

    pub fn start(&self) -> usize {
        self.seq[0]
    }

    fn next(&mut self) -> Option<usize> {
        match self.seq.get(self.index) {
            Some(next) => {
                self.index += 1;
                Some(*next)
            }
            None => match self.mode {
                AnimationMode::Repeat => {
                    self.index = 0;
                    self.seq.get(self.index).copied()
                }
                AnimationMode::Once | AnimationMode::Despawn => None,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    Repeat,
    Once,
    Despawn,
}

fn init_atlas_index(
    mut query: Query<(&mut Sprite, &AnimationController), Added<AnimationController>>,
) {
    for (mut sprite, controller) in query.iter_mut() {
        sprite.texture_atlas.as_mut().unwrap().index = controller.indices.start();
    }
}

fn animate_sprites(
    mut commands: Commands,
    time: Res<Time<Physics>>,
    mut query: Query<(Entity, &mut AnimationController, &mut Sprite)>,
) {
    for (entity, mut controller, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        controller.timer.tick(time.delta());
        if controller.timer.just_finished() {
            if let Some(index) = controller.indices.next() {
                atlas.index = index;
            } else if controller.indices.mode == AnimationMode::Despawn {
                commands.entity(entity).despawn();
            }
        }
    }
}
