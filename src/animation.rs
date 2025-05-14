use bevy::platform::collections::HashMap;
use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LayoutHash::default())
            .add_systems(
                PreUpdate,
                (insert_animation_controller, init_atlas_index).chain(),
            )
            .add_systems(Update, (animate_sprites, flip_sprites));
    }
}

pub trait AnimationAppExt {
    fn register_layout(&mut self, path: &'static str, layout: TextureAtlasLayout) -> &mut Self;
}

impl AnimationAppExt for App {
    fn register_layout(&mut self, path: &'static str, layout: TextureAtlasLayout) -> &mut Self {
        self.add_systems(
            PreStartup,
            move |mut hash: ResMut<LayoutHash>, mut layouts: ResMut<Assets<TextureAtlasLayout>>| {
                hash.0.insert(path, layouts.add(layout.clone()));
            },
        )
    }
}

#[derive(Component)]
pub struct AnimationSprite {
    path: &'static str,
    indices: AnimationIndices,
    interval: f32,
}

impl AnimationSprite {
    pub fn once(
        path: &'static str,
        interval: f32,
        indices: impl IntoIterator<Item = usize>,
    ) -> Self {
        Self {
            path,
            indices: AnimationIndices::once_despawn(indices),
            interval,
        }
    }

    pub fn repeating(
        path: &'static str,
        interval: f32,
        indices: impl IntoIterator<Item = usize>,
    ) -> Self {
        Self {
            path,
            indices: AnimationIndices::repeating(indices),
            interval,
        }
    }
}

#[derive(Default, Resource)]
struct LayoutHash(HashMap<&'static str, Handle<TextureAtlasLayout>>);

fn insert_animation_controller(
    mut commands: Commands,
    server: Res<AssetServer>,
    sprites: Query<(Entity, &AnimationSprite)>,
    layouts: Res<LayoutHash>,
) {
    for (entity, sprite) in sprites.iter() {
        if let Some(layout) = layouts.0.get(sprite.path).cloned() {
            commands.entity(entity).insert((
                AnimationController::from_seconds(sprite.indices.clone(), sprite.interval),
                Sprite::from_atlas_image(
                    server.load(sprite.path),
                    TextureAtlas { layout, index: 0 },
                ),
            ));
        } else {
            error!("Layout not registered for path: {}", sprite.path);
        }

        commands.entity(entity).remove::<AnimationSprite>();
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

#[derive(Clone, Component)]
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
    time: Res<Time>,
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

#[derive(Default, Component)]
pub struct FlipX;

#[derive(Default, Component)]
pub struct FlipY;

fn flip_sprites(
    mut commands: Commands,
    mut sprites: Query<
        (Entity, &mut Sprite, Option<&FlipY>, Option<&FlipX>),
        Or<(With<FlipX>, With<FlipY>)>,
    >,
) {
    for (entity, mut sprite, y, x) in sprites.iter_mut() {
        if y.is_some() {
            commands.entity(entity).remove::<FlipY>();
            sprite.flip_y = true;
        }

        if x.is_some() {
            commands.entity(entity).remove::<FlipX>();
            sprite.flip_x = true;
        }
    }
}
