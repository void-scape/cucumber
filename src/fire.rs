use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FireMaterial>::default())
            .add_systems(Startup, scrolling_texture)
            .add_systems(Update, update_scrolling_background);
    }
}

#[derive(Clone, Asset, TypePath, AsBindGroup)]
struct FireMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    uv_offset: f32,
}

impl Material2d for FireMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fire.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
struct Speed(f32);

fn scrolling_texture(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FireMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(256. / 8., 256. / 8.))),
        MeshMaterial2d(materials.add(FireMaterial {
            texture: server.load_with_settings("noise_texture.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
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
        //HIGH_RES_LAYER,
        Speed(0.4),
        Transform::from_xyz(0., 0., -999.),
    ));
}

fn update_scrolling_background(
    query: Query<(&MeshMaterial2d<FireMaterial>, &Speed)>,
    mut materials: ResMut<Assets<FireMaterial>>,
    time: Res<Time>,
) {
    for (handle, speed) in query.iter() {
        let material = materials.get_mut(&handle.0).unwrap();
        material.uv_offset += speed.0 * time.delta_secs();
        if material.uv_offset >= 1. {
            material.uv_offset = 0.;
        }
    }
}
