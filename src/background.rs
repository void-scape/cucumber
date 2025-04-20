use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy_pixel_gfx::pixel_perfect::HIGH_RES_BACKGROUND_LAYER;

const SCROLL_SPEED: f32 = 0.3;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ScrollingTexture>::default())
            .add_systems(Startup, scrolling_background)
            .add_systems(Update, update_scrolling_background);
        //app.add_plugins(MandelbrotPlugin);
    }
}

fn scrolling_background(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<ScrollingTexture>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(320., 180.))),
        MeshMaterial2d(custom_materials.add(ScrollingTexture {
            texture: server.load_with_settings("background.png", |s: &mut _| {
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
        Transform::from_xyz(0., 0., -999.),
    ));
}

fn update_scrolling_background(
    query: Query<&MeshMaterial2d<ScrollingTexture>>,
    mut materials: ResMut<Assets<ScrollingTexture>>,
    time: Res<Time>,
) {
    for handle in query.iter() {
        let material = materials.get_mut(&handle.0).unwrap();
        material.uv_offset -= SCROLL_SPEED * time.delta_secs();
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
}

pub struct MandelbrotPlugin;

impl Plugin for MandelbrotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<MandelbrotMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update_shader_params);
    }
}

#[derive(Debug, Clone, AsBindGroup, Asset, TypePath)]
struct MandelbrotMaterial {
    #[uniform(0)]
    params: MandelbrotParams,
}

#[derive(Clone, Debug, Default, ShaderType)]
struct MandelbrotParams {
    center: Vec2,
    zoom: f32,
    aspect_ratio: f32,
    max_iterations: u32,
    color_shift: f32,
}

impl Material2d for MandelbrotMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mandelbrot.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut windows: Query<&mut Window>,
) {
    let window = windows.single_mut();
    let aspect_ratio = window.width() / window.height();

    let material = materials.add(MandelbrotMaterial {
        params: MandelbrotParams {
            center: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            aspect_ratio,
            max_iterations: 100,
            color_shift: 0.0,
        },
    });

    let quad = meshes.add(Rectangle::new(window.width(), window.height()));
    commands.spawn((
        MeshMaterial2d(material),
        Mesh2d(quad),
        HIGH_RES_BACKGROUND_LAYER,
    ));
}

fn update_shader_params(
    time: Res<Time>,
    mut input: EventReader<KeyboardInput>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    mut material_query: Query<&MeshMaterial2d<MandelbrotMaterial>>,
    mut windows: Query<&mut Window>,
) {
    let window = windows.single_mut();
    let aspect_ratio = window.width() / window.height();

    let Some(material_handle) = material_query.iter_mut().next() else {
        return;
    };
    let Some(material) = materials.get_mut(&material_handle.0) else {
        return;
    };

    for event in input.read() {
        material.params.aspect_ratio = aspect_ratio;
        material.params.color_shift = time.elapsed_secs() * 0.2;
        let mut center = material.params.center;
        let mut zoom = material.params.zoom;

        let move_speed = 0.5 / zoom;

        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::KeyW => center.y -= move_speed * time.delta_secs(),
                KeyCode::KeyS => center.y += move_speed * time.delta_secs(),
                KeyCode::KeyA => center.x -= move_speed * time.delta_secs(),
                KeyCode::KeyD => center.x += move_speed * time.delta_secs(),
                KeyCode::KeyE => zoom *= 1.0 + time.delta_secs(),
                KeyCode::KeyQ => zoom *= 1.0 - time.delta_secs(),
                _ => {}
            }

            if !event.repeat {
                match event.key_code {
                    KeyCode::Digit1 => {
                        material.params.max_iterations =
                            (material.params.max_iterations - 50).max(50)
                    }
                    KeyCode::Digit2 => {
                        material.params.max_iterations = material.params.max_iterations + 50
                    }
                    _ => {}
                }
            }
        }

        material.params.center = center;
        material.params.zoom = zoom;
    }
}
