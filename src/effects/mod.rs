use crate::animation::{AnimationAppExt, AnimationSprite, FlipX, FlipY};
use crate::assets::MISC_PATH;
use crate::health::Dead;
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_seedling::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnExplosion>()
            .add_systems(
                Update,
                (
                    spawn_explosions,
                    write_explosions,
                    (spawn_blasters, update_blasters).chain(),
                ),
            )
            .register_layout(
                "fire_sparks.png",
                TextureAtlasLayout::from_grid(UVec2::splat(96), 4, 5, None, None),
            )
            .register_layout(
                "sparks.png",
                TextureAtlasLayout::from_grid(UVec2::splat(150), 5, 6, None, None),
            )
            .register_layout(
                "explosion2.png",
                TextureAtlasLayout::from_grid(UVec2::splat(64), 12, 9, None, None),
            )
            .register_layout(
                "explosion3.png",
                TextureAtlasLayout::from_grid(UVec2::splat(64), 12, 9, None, None),
            );
    }
}

fn write_explosions(
    mut writer: EventWriter<SpawnExplosion>,
    explosions: Query<(&GlobalTransform, &Explosion), With<Dead>>,
) {
    for (gt, explosion) in explosions.iter() {
        writer.write(SpawnExplosion {
            position: gt.translation().xy(),
            explosion: *explosion,
        });
    }
}

#[derive(Clone, Copy, Event)]
pub struct SpawnExplosion {
    pub position: Vec2,
    pub explosion: Explosion,
}

#[derive(Clone, Copy, PartialEq, Eq, Component)]
pub enum Explosion {
    Big,
    Small,
}

fn spawn_explosions(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<SpawnExplosion>,
) {
    let mut big = false;
    let mut small = false;
    for event in reader.read() {
        match event.explosion {
            Explosion::Big => {
                big = true;
                commands.spawn((
                    Transform::from_translation(event.position.extend(-97.)),
                    AnimationSprite::once("explosion2.png", 0.04, 0..=11),
                ));
                commands.spawn((
                    Transform::from_translation((event.position + Vec2::new(5., -5.)).extend(-98.)),
                    AnimationSprite::once("explosion2.png", 0.08, 0..=11),
                    FlipX,
                ));
                commands.spawn((
                    Transform::from_translation((event.position + Vec2::new(-5., 5.)).extend(-99.)),
                    AnimationSprite::once("explosion2.png", 0.1, 0..=11),
                    FlipY,
                ));

                commands.spawn((
                    ParticleSpawner::default(),
                    ParticleEffectHandle(server.load("particles/embers.ron")),
                    OneShot::Despawn,
                    Transform::from_translation(event.position.extend(-100.)),
                ));
            }
            Explosion::Small => {
                small = true;
                commands.spawn((
                    Transform::from_translation(event.position.extend(-97.))
                        .with_scale(Vec3::splat(0.75)),
                    AnimationSprite::once("explosion2.png", 0.04, 0..=11),
                ));
            }
        }
    }

    if small || big {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/explosion2.wav")),
            PitchRange(0.8..1.2),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));
    }

    if big {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/explosion1.wav")),
            PitchRange(0.8..1.2),
            PlaybackSettings {
                volume: Volume::Linear(0.25),
                ..PlaybackSettings::ONCE
            },
        ));
    }
}

#[derive(Component)]
pub struct Blasters(pub &'static [Vec3]);

#[derive(Component)]
struct BlastersEntity;

fn spawn_blasters(
    mut commands: Commands,
    blasters: Query<(Entity, &Blasters, Option<&Children>), Added<Blasters>>,
    entities: Query<Entity, With<BlastersEntity>>,
) {
    for (entity, blasters, children) in blasters.iter() {
        if let Some(children) = children {
            for entity in entities.iter_many(children) {
                commands.entity(entity).despawn();
            }
        }

        commands.entity(entity).with_children(|root| {
            for t in blasters.0.iter().copied() {
                root.spawn((
                    BlastersEntity,
                    Visibility::Hidden,
                    Transform::from_translation(t),
                    AnimationSprite::repeating(MISC_PATH, 0.1, 18..=21),
                ));
            }
        });
    }
}

fn update_blasters(
    blasters: Query<(&LinearVelocity, &Children), (With<Blasters>, Changed<LinearVelocity>)>,
    mut vis: Query<&mut Visibility, With<BlastersEntity>>,
) {
    for (velocity, children) in blasters.iter() {
        let mut iter = vis.iter_many_mut(children);
        while let Some(mut vis) = iter.fetch_next() {
            if velocity.0.y > 1. {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}
