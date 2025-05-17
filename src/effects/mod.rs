use crate::animation::{AnimationAppExt, AnimationSprite, FlipX, FlipY};
use crate::assets::MISC_PATH;
use crate::health::Dead;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy_enoki::prelude::*;
use bevy_seedling::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<Lightning>::default())
            .add_event::<SpawnExplosion>()
            //.add_systems(Startup, lightning)
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
                    Transform::from_translation(event.position.extend(-96.)),
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

#[derive(Default, Component)]
pub struct AlwaysBlast;

#[derive(Component)]
struct BlastersEntity;

fn spawn_blasters(mut commands: Commands, blasters: Query<(Entity, &Blasters), Added<Blasters>>) {
    for (entity, blasters) in blasters.iter() {
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
    blasters: Query<
        (&LinearVelocity, &Children, Option<&AlwaysBlast>),
        (With<Blasters>, Changed<LinearVelocity>),
    >,
    mut vis: Query<&mut Visibility, With<BlastersEntity>>,
) {
    for (velocity, children, always) in blasters.iter() {
        let mut iter = vis.iter_many_mut(children);
        while let Some(mut vis) = iter.fetch_next() {
            if always.is_some() {
                *vis = Visibility::Visible;
            } else {
                if velocity.0.y > 1. {
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}

#[derive(Clone, Asset, TypePath, AsBindGroup)]
struct Lightning {
    #[uniform(0)]
    uniform: LightningUniform,
    //#[texture(0)]
    //#[sampler(1)]
    //texture: Handle<Image>,
    //#[uniform(2)]
    //uv_offset: f32,
    //#[uniform(3)]
    //alpha: f32,
    //#[uniform(4)]
    //alpha_effect: f32,
}

fn lightning(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<Lightning>>,
) {
    let width = 200.;
    let height = 200.;
    commands.spawn((
        //HIGH_RES_LAYER,
        Mesh2d(meshes.add(Rectangle::new(width, height))),
        MeshMaterial2d(mats.add(Lightning {
            uniform: LightningUniform {
                resolution: Vec2::new(width, height),
                intensity: 2.,
                branches: 0.3,
                color: Color::srgb_u8(120, 215, 255).to_srgba().to_vec3(),
                origin: Vec2::new(0.5, 1.),
                target: Vec2::new(0.5, 0.),
                width: 0.05,
            },
        })),
    ));
}

#[derive(Clone, ShaderType)]
struct LightningUniform {
    resolution: Vec2,
    intensity: f32,
    branches: f32,
    color: Vec3,
    origin: Vec2,
    target: Vec2,
    width: f32,
}

impl Material2d for Lightning {
    fn fragment_shader() -> ShaderRef {
        "shaders/lightning.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
