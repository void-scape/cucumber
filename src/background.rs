use crate::DespawnRestart;
use crate::bullet::emitter::{BackgroundGattlingEmitter, BulletModifiers, Rate};
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use rand::Rng;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ScrollingTexture>::default())
            .insert_resource(BackgroundEmitters(true))
            .add_systems(Startup, background)
            .add_systems(
                Update,
                (
                    update_scrolling_background,
                    spawn_background_emitters,
                    update_emitters,
                ),
            );
    }
}

pub const LAYER1: f32 = -10.;
pub const LAYER2: f32 = -250.;
pub const LAYER3: f32 = -500.;
pub const LAYER4: f32 = -750.;
pub const LAYER5: f32 = -999.;

const SPEED: f32 = 0.5;
const SCROLL_SPEED1: f32 = 1. * SPEED;
const SCROLL_SPEED2: f32 = 0.5 * SPEED;
const SCROLL_SPEED3: f32 = 0.1 * SPEED;

/// Enable/disable background emitter spawning
#[derive(Resource)]
pub struct BackgroundEmitters(pub bool);

#[derive(Component)]
struct EmitterLifespan(Timer);

fn spawn_background_emitters(
    mut commands: Commands,
    time: Res<Time>,
    emitters: Res<BackgroundEmitters>,
    mut timer: Local<Option<Timer>>,
    mut root: Local<Option<Entity>>,
) {
    if !emitters.0 {
        return;
    }

    let mut entity = root.get_or_insert_with(|| {
        commands
            .spawn((Transform::default(), Visibility::Visible, DespawnRestart))
            .id()
    });
    if commands.get_entity(*entity).is_err() {
        entity = root.insert(
            commands
                .spawn((Transform::default(), Visibility::Visible, DespawnRestart))
                .id(),
        );
    }

    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.5, TimerMode::Repeating));
    timer.tick(time.delta());

    let mut rng = rand::rng();
    if timer.just_finished() && rng.random() {
        let z = match rng.random_range(0..=1) {
            0 => LAYER1 - 5.,
            1 => LAYER2 - 5.,
            _ => unreachable!(),
        };

        let w = crate::WIDTH / 2.;
        let h = crate::HEIGHT / 2.;
        const ANGLE_RAND: f32 = 0.4;
        const OFFSET: f32 = 15.;

        let (xy, rot) = match rng.random_range(0..4) {
            // top
            0 => (
                Vec2::new(rng.random_range(-w..w), -h - OFFSET),
                Vec2::Y + Vec2::new(rng.random_range(-ANGLE_RAND..ANGLE_RAND), 0.),
            ),
            // bottom
            1 => (
                Vec2::new(rng.random_range(-w..w), h + OFFSET),
                Vec2::NEG_Y + Vec2::new(rng.random_range(-ANGLE_RAND..ANGLE_RAND), 0.),
            ),
            // left
            2 => (
                Vec2::new(-w - OFFSET, rng.random_range(-h..h)),
                Vec2::X + Vec2::new(0., rng.random_range(-ANGLE_RAND..ANGLE_RAND)),
            ),
            // right
            3 => (
                Vec2::new(w + OFFSET, rng.random_range(-h..h)),
                Vec2::NEG_X + Vec2::new(0., rng.random_range(-ANGLE_RAND..ANGLE_RAND)),
            ),
            _ => unreachable!(),
        };

        commands.entity(*entity).with_child((
            BackgroundGattlingEmitter(0.1, rot),
            EmitterLifespan(Timer::from_seconds(
                rng.random_range(1.0..3.0),
                TimerMode::Once,
            )),
            BulletModifiers {
                rate: Rate::Factor(rng.random_range(0.75..1.25)),
                speed: rng.random_range(0.25..0.75),
                ..Default::default()
            },
            Transform::from_translation(xy.extend(z)),
        ));
    }
}

fn update_emitters(
    mut commands: Commands,
    time: Res<Time>,
    mut emitters: Query<(Entity, &mut EmitterLifespan)>,
) {
    let delta = time.delta();
    for (entity, mut lifespan) in emitters.iter_mut() {
        lifespan.0.tick(delta);
        if lifespan.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct Speed(f32);

fn background(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ScrollingTexture>>,
) {
    commands.spawn((
        Sprite::from_image(server.load("space.png")),
        Transform::from_xyz(0., 0., LAYER5),
    ));

    let mesh = meshes.add(Rectangle::new(crate::WIDTH, crate::HEIGHT));

    //first
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED1,
        Vec3::new(-30., 0., LAYER1),
        0.8,
        1.,
        0.,
    );
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED1,
        Vec3::new(30., 0., LAYER1 - 0.1),
        0.8,
        1.,
        0.2,
    );
    // second
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED2,
        Vec3::new(-45., 0., LAYER2),
        0.95,
        0.25,
        0.3,
    );
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED2,
        Vec3::new(45., 0., LAYER2 - 0.5),
        0.95,
        0.25,
        0.7,
    );
    // third
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED3,
        Vec3::new(crate::WIDTH / 2., 0., LAYER3),
        1.,
        0.,
        0.8,
    );
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED3,
        Vec3::new(-crate::WIDTH / 2., 0., LAYER3 - 0.1),
        1.,
        0.,
        0.15,
    );
    spawn_clouds(
        &mut commands,
        &server,
        &mut mats,
        &mesh,
        SCROLL_SPEED3,
        Vec3::new(0., 0., LAYER3 - 0.2),
        1.,
        0.,
        0.55,
    );
}

fn spawn_clouds(
    commands: &mut Commands,
    server: &AssetServer,
    mats: &mut Assets<ScrollingTexture>,
    mesh: &Handle<Mesh>,
    speed: f32,
    position: Vec3,
    alpha: f32,
    fade_strength: f32,
    uv_offset: f32,
) {
    commands.spawn((
        Mesh2d(mesh.clone()),
        MeshMaterial2d(mats.add(ScrollingTexture {
            alpha,
            alpha_effect: fade_strength,
            texture: server.load_with_settings("clouds.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::MirrorRepeat,
                        address_mode_v: ImageAddressMode::MirrorRepeat,
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
            uv_offset,
        })),
        Speed(speed),
        Transform::from_translation(position),
    ));
}

fn update_scrolling_background(
    query: Query<(&MeshMaterial2d<ScrollingTexture>, &Speed)>,
    mut materials: ResMut<Assets<ScrollingTexture>>,
    time: Res<Time>,
) {
    for (handle, speed) in query.iter() {
        let material = materials.get_mut(&handle.0).unwrap();
        material.uv_offset -= speed.0 * time.delta_secs();
        if material.uv_offset >= 1. {
            material.uv_offset = 0.;
        }
    }
}

#[derive(Clone, Asset, TypePath, AsBindGroup)]
struct ScrollingTexture {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    uv_offset: f32,
    #[uniform(3)]
    alpha: f32,
    #[uniform(4)]
    alpha_effect: f32,
}

impl Material2d for ScrollingTexture {
    fn fragment_shader() -> ShaderRef {
        "shaders/scrolling_texture.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
