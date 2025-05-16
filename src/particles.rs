use std::marker::PhantomData;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_enoki::prelude::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParticleSpriteHash::default());
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ParticleSet;

pub trait ParticleState: Component {
    fn enabled(&self) -> bool;
}

pub trait ParticleAppExt {
    fn register_particle_state<S: ParticleState>(&mut self) -> &mut Self;
}

impl ParticleAppExt for App {
    fn register_particle_state<S: ParticleState>(&mut self) -> &mut Self {
        self.add_systems(
            PostUpdate,
            (spawn_particles::<S>, update_particles::<S>).in_set(ParticleSet),
        )
    }
}

#[derive(Component)]
pub struct ParticleBundle<S>(Vec<ParticleEmitter>, PhantomData<fn() -> S>);

impl<S> ParticleBundle<S>
where
    S: ParticleState,
{
    pub fn new(spawners: impl IntoIterator<Item = ParticleEmitter>) -> Self {
        Self(spawners.into_iter().collect(), PhantomData)
    }

    pub fn from_emitter(emitter: ParticleEmitter) -> Self {
        Self(vec![emitter], PhantomData)
    }
}

pub struct ParticleEmitter {
    sprite_material: Option<(&'static str, Option<(u32, u32)>)>,
    effect: &'static str,
    spawn_with: Vec<Box<dyn FnMut(&mut EntityCommands) + Send + Sync>>,
}

impl ParticleEmitter {
    pub fn from_effect(path: &'static str) -> Self {
        Self {
            effect: path,
            sprite_material: None,
            spawn_with: Vec::new(),
        }
    }

    pub fn with_sprite(mut self, path: &'static str) -> Self {
        self.sprite_material = Some((path, None));
        self
    }

    pub fn with_animated_sprite(
        mut self,
        path: &'static str,
        max_hframes: u32,
        max_vframes: u32,
    ) -> Self {
        self.sprite_material = Some((path, Some((max_hframes, max_vframes))));
        self
    }

    pub fn with(
        mut self,
        commands: impl FnMut(&mut EntityCommands) + Send + Sync + 'static,
    ) -> Self {
        self.spawn_with.push(Box::new(commands));
        self
    }
}

pub fn transform(transform: Transform) -> impl FnMut(&mut EntityCommands) {
    move |commands: &mut EntityCommands| {
        commands.insert(transform);
    }
}

#[derive(Default, Resource)]
struct ParticleSpriteHash(HashMap<SpriteMat, Handle<SpriteParticle2dMaterial>>);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SpriteMat {
    Animated(&'static str),
    Static(&'static str),
}

fn spawn_particles<S: ParticleState>(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut emitters: Query<(Entity, &S, &mut ParticleBundle<S>), Added<ParticleBundle<S>>>,
    mut mats: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut particle_sprite_hash: ResMut<ParticleSpriteHash>,
) {
    for (entity, state, mut particles) in emitters.iter_mut() {
        for spawner in particles.0.iter_mut() {
            let mut commands = if let Some((mat_path, frames)) = spawner.sprite_material {
                let new_entity = commands
                    .spawn((
                        ChildOf(entity),
                        ParticleSpawnerState {
                            active: state.enabled(),
                            ..Default::default()
                        },
                        ParticleEffectHandle(server.load(spawner.effect)),
                    ))
                    .id();

                match frames {
                    Some((h, v)) => {
                        commands.entity(new_entity).insert(ParticleSpawner(
                            particle_sprite_hash
                                .0
                                .entry(SpriteMat::Animated(mat_path))
                                .or_insert_with(|| {
                                    mats.add(SpriteParticle2dMaterial::new(
                                        server.load(mat_path),
                                        h,
                                        v,
                                    ))
                                })
                                .clone(),
                        ));
                    }
                    None => {
                        commands.entity(new_entity).insert(ParticleSpawner(
                            particle_sprite_hash
                                .0
                                .entry(SpriteMat::Static(mat_path))
                                .or_insert_with(|| {
                                    mats.add(SpriteParticle2dMaterial::from_texture(
                                        server.load(mat_path),
                                    ))
                                })
                                .clone(),
                        ));
                    }
                };

                commands.entity(new_entity)
            } else {
                commands.spawn((
                    ChildOf(entity),
                    ParticleSpawnerState {
                        active: state.enabled(),
                        ..Default::default()
                    },
                    ParticleEffectHandle(server.load(spawner.effect)),
                    ParticleSpawner::default(),
                ))
            };

            for spawn_mod in spawner.spawn_with.iter_mut() {
                (spawn_mod)(&mut commands);
            }
        }
    }
}

fn update_particles<S: ParticleState>(
    emitters: Query<(&S, &Children), (Changed<S>, With<ParticleBundle<S>>)>,
    mut spawners: Query<&mut ParticleSpawnerState>,
) {
    for (emitter, children) in emitters.iter() {
        let mut iter = spawners.iter_many_mut(children);
        while let Some(mut state) = iter.fetch_next() {
            state.active = emitter.enabled();
        }
    }
}
