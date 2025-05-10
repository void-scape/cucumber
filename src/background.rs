use avian2d::prelude::Physics;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ScrollingTexture>::default())
            .add_systems(Startup, scrolling_background)
            .add_systems(Update, update_scrolling_background);
    }
}

const BACKGROUND_WIDTH: f32 = 128.;
const BACKGROUND_HEIGHT: f32 = 256.;
const BACKGROUND_PATH1: &'static str = "shooters/background1.png";
const BACKGROUND_PATH2: &'static str = "shooters/background2.png";
const SCROLL_SPEED1: f32 = 0.1;
const SCROLL_SPEED2: f32 = 0.15;

#[derive(Component)]
struct Speed(f32);

fn scrolling_background(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<ScrollingTexture>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(BACKGROUND_WIDTH, BACKGROUND_HEIGHT))),
        MeshMaterial2d(custom_materials.add(ScrollingTexture {
            texture: server.load_with_settings(BACKGROUND_PATH1, |s: &mut _| {
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
            uv_offset: 0.,
        })),
        Speed(SCROLL_SPEED1),
        Transform::from_xyz(0., 0., -1.),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(BACKGROUND_WIDTH, BACKGROUND_HEIGHT))),
        MeshMaterial2d(custom_materials.add(ScrollingTexture {
            texture: server.load_with_settings(BACKGROUND_PATH2, |s: &mut _| {
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
            uv_offset: 0.,
        })),
        Speed(SCROLL_SPEED2),
        Transform::from_xyz(0., 0., -1.5),
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
}

impl Material2d for ScrollingTexture {
    fn fragment_shader() -> ShaderRef {
        "shaders/scrolling_texture.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
