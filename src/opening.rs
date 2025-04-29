use crate::characters::Character;
use crate::{GameState, RES_HEIGHT, RES_WIDTH, RESOLUTION_SCALE};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{Anchor, Material2d, Material2dPlugin};
use bevy::text::TextBounds;
use bevy_optix::glitch::{GlitchPlugin, GlitchSettings};
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};
use bevy_optix::post_process::PostProcessCommand;
use bevy_pretty_text::prelude::SfxRate;
use bevy_pretty_text::prelude::*;
use bevy_seedling::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use bevy_sequence::prelude::*;
use bevy_textbox::{Continue, TextBox, TextBoxEntity};
use bevy_tween::combinator::{sequence, tween};
use bevy_tween::interpolate::sprite_color;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Interpolator, Repeat, RepeatStyle};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};
use std::time::Duration;

pub struct OpeningPlugin;

impl Plugin for OpeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GlitchPlugin, MandelbrotPlugin))
            .add_systems(OnEnter(GameState::Opening), begin);
    }
}

#[derive(Component)]
struct OpeningEntity;

fn begin(mut commands: Commands, server: Res<AssetServer>) {
    commands.post_process::<OuterCamera>(GlitchSettings::from_intensity(0.3));

    let mask = commands
        .spawn((
            OpeningEntity,
            Sprite::from_color(
                Color::linear_rgba(0., 0., 0., 1.),
                Vec2::new(RES_WIDTH * RESOLUTION_SCALE, RES_HEIGHT * RESOLUTION_SCALE),
            ),
            Transform::from_xyz(0., 0., -2.),
        ))
        .id();
    commands
        .animation()
        .repeat(Repeat::infinitely())
        .insert(sequence((
            tween(
                Duration::from_secs_f32(0.1),
                EaseKind::BackInOut,
                mask.into_target().with(sprite_color(
                    Color::linear_rgba(0., 0., 0., 1.),
                    Color::linear_rgba(0., 0., 0., 0.),
                )),
            ),
            tween(
                Duration::from_secs_f32(0.2),
                EaseKind::BackInOut,
                mask.into_target().with(sprite_color(
                    Color::linear_rgba(0., 0., 0., 0.),
                    Color::linear_rgba(0., 0., 0., 1.),
                )),
            ),
            tween(
                Duration::from_secs_f32(0.8),
                EaseKind::Linear,
                mask.into_target().with(sprite_color(
                    Color::linear_rgba(0., 0., 0., 1.),
                    Color::linear_rgba(0., 0., 0., 1.),
                )),
            ),
            tween(
                Duration::from_secs_f32(1.2),
                EaseKind::BackInOut,
                mask.into_target().with(sprite_color(
                    Color::linear_rgba(0., 0., 0., 1.),
                    Color::linear_rgba(0., 0., 0., 0.),
                )),
            ),
            tween(
                Duration::from_secs_f32(1.),
                EaseKind::Linear,
                mask.into_target().with(sprite_color(
                    Color::linear_rgba(0., 0., 0., 1.),
                    Color::linear_rgba(0., 0., 0., 1.),
                )),
            ),
        )))
        .insert(OpeningEntity);

    // cutting the next two calls short makes this disappear after text appears????
    commands
        .spawn((
            OpeningEntity,
            SamplePlayer::new(server.load("audio/drone.wav")),
            PlaybackSettings::LOOP,
        ))
        .effect(VolumeNode {
            volume: Volume::Linear(0.5),
        });

    // 3.
    run_after(
        Duration::from_secs_f32(0.),
        |mut commands: Commands, server: Res<AssetServer>| {
            commands
                .spawn((
                    SamplePlayer::new(server.load("audio/sfx/note2.wav")),
                    PlaybackSettings::ONCE,
                ))
                .effect(VolumeNode {
                    volume: Volume::Linear(0.5),
                });
        },
        &mut commands,
    );
    // 4.
    run_after(
        Duration::from_secs_f32(0.),
        corrupted_message,
        &mut commands,
    );
}

fn corrupted_message(mut commands: Commands, server: Res<AssetServer>) {
    let textbox = TextBox::new((
        TextBounds::new(256., 128.),
        TextFont {
            font_size: 18.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Anchor::TopLeft,
        TextLayout::default().with_linebreak(LineBreak::WordBoundary),
        Transform::from_scale(Vec3::splat(1. / RESOLUTION_SCALE))
            .with_translation(Vec3::new(-32., -85., 100.)),
    ));

    let textbox = commands
        .spawn((
            OpeningEntity,
            HIGH_RES_LAYER,
            textbox,
            Sprite::from_image(server.load("textbox.png")),
            SfxRate::new(
                0.1,
                (
                    SamplePlayer::new(server.load("audio/sfx/beep.wav")),
                    PitchRange(0.99..1.01),
                    PlaybackSettings {
                        volume: Volume::Linear(0.2),
                        ..PlaybackSettings::ONCE
                    },
                ),
            ),
            Transform::from_translation(Vec3::default().with_z(100.))
                .with_scale(Vec3::splat(RESOLUTION_SCALE)),
        ))
        .with_child((
            Sprite {
                image: server.load("continue.png"),
                anchor: Anchor::TopLeft,
                ..Default::default()
            },
            Transform::from_translation(Vec3::default().with_z(100.))
                .with_scale(Vec3::splat(RESOLUTION_SCALE)),
            Continue,
            Visibility::Hidden,
        ))
        .with_child((
            Character::Noise,
            Transform::from_xyz(-47., -98., 100.).with_scale(Vec3::splat(RESOLUTION_SCALE)),
        ))
        .with_child((SamplePlayer::new(server.load("audio/sfx/glitch.wav")), {
            let mut settings = PlaybackSettings::LOOP;
            settings.volume = Volume::Linear(0.4);
            settings
        }))
        .id();

    commands.spawn((
        SamplePlayer::new(server.load("audio/sfx/note1.wav")),
        PlaybackSettings {
            volume: Volume::Linear(0.2),
            ..PlaybackSettings::ONCE
        },
    ));

    // 1.
    run_after(
        Duration::from_secs_f32(0.),
        message_contents(textbox),
        &mut commands,
    );
}

fn message_contents(entity: Entity) -> impl Fn(Commands) {
    move |mut commands: Commands| {
        let frag = (
            "01001110 01000101 01010111 00100000 01000100 01001001 01001101",
            "01000101 01001110 01010011 01001001 01001111 01001110",
            s!("<0.5>...<1.2>"),
            "RUN!",
        )
            .always()
            .once()
            .on_end(end);
        spawn_root_with(frag, &mut commands, TextBoxEntity::new(entity));
    }
}

fn end(mut commands: Commands, opening_entities: Query<Entity, With<OpeningEntity>>) {
    commands.remove_post_process::<GlitchSettings, OuterCamera>();
    for entity in opening_entities.iter() {
        commands.entity(entity).despawn();
    }
    commands.set_state(GameState::Game);
}

pub struct MandelbrotPlugin;

impl Plugin for MandelbrotPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<MandelbrotMaterial>::default())
            .add_systems(OnEnter(GameState::Opening), setup)
            .add_systems(
                Update,
                update_mandelbrot_zoom.run_if(in_state(GameState::Opening)),
            )
            .add_tween_systems(component_tween_system::<TweenMandelbrotZoom>());
        //.add_systems(Update, update_shader_params);
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
) -> Result {
    let window = windows.single_mut()?;
    let material = materials.add(MandelbrotMaterial {
        params: MandelbrotParams {
            center: Vec2::new(0.38870746, -0.13461724),
            zoom: 18479.582,
            aspect_ratio: 0.625,
            max_iterations: 150,
            color_shift: 18.097664,
        },
    });

    let quad = meshes.add(Rectangle::new(window.width(), window.height()));
    let mandelbrot = commands
        .spawn((
            OpeningEntity,
            MandelbrotZoom::default(),
            MeshMaterial2d(material),
            Mesh2d(quad),
            Transform::from_xyz(0., 0., -3.),
        ))
        .id();

    commands
        .animation()
        .repeat_style(RepeatStyle::WrapAround)
        .insert(tween(
            Duration::from_secs_f32(2.),
            EaseKind::QuarticOut,
            mandelbrot
                .into_target()
                .with(mandelbrot_zoom(122.64574, 18479.582)),
        ))
        .insert(OpeningEntity);

    Ok(())
}

fn update_mandelbrot_zoom(
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    query: Query<(&MeshMaterial2d<MandelbrotMaterial>, &MandelbrotZoom)>,
) {
    for (handle, zoom) in query.iter() {
        let Some(material) = materials.get_mut(&handle.0) else {
            continue;
        };
        material.params.zoom = zoom.0;
    }
}

//fn update_shader_params(
//    time: Res<Time>,
//    mut input: EventReader<KeyboardInput>,
//    mut materials: ResMut<Assets<MandelbrotMaterial>>,
//    mut material_query: Query<&MeshMaterial2d<MandelbrotMaterial>>,
//    mut windows: Query<&mut Window>,
//) -> Result {
//    let window = windows.single_mut()?;
//    let aspect_ratio = window.width() / window.height();
//
//    let Some(material_handle) = material_query.iter_mut().next() else {
//        return Ok(());
//    };
//    let Some(material) = materials.get_mut(&material_handle.0) else {
//        return Ok(());
//    };
//
//    for event in input.read() {
//        material.params.aspect_ratio = aspect_ratio;
//        material.params.color_shift = time.elapsed_secs() * 0.2;
//        let mut center = material.params.center;
//        let mut zoom = material.params.zoom;
//
//        let move_speed = 0.5 / zoom;
//
//        if event.state.is_pressed() {
//            match event.key_code {
//                KeyCode::KeyW => center.y -= move_speed * time.delta_secs(),
//                KeyCode::KeyS => center.y += move_speed * time.delta_secs(),
//                KeyCode::KeyA => center.x -= move_speed * time.delta_secs(),
//                KeyCode::KeyD => center.x += move_speed * time.delta_secs(),
//                KeyCode::KeyE => zoom *= 1.0 + time.delta_secs(),
//                KeyCode::KeyQ => zoom *= 1.0 - time.delta_secs(),
//                _ => {}
//            }
//
//            if !event.repeat {
//                match event.key_code {
//                    KeyCode::Digit1 => {
//                        material.params.max_iterations =
//                            (material.params.max_iterations - 50).max(50)
//                    }
//                    KeyCode::Digit2 => {
//                        material.params.max_iterations = material.params.max_iterations + 50
//                    }
//                    _ => {}
//                }
//            }
//        }
//
//        material.params.center = center;
//        material.params.zoom = zoom;
//    }
//
//    Ok(())
//}

#[derive(Default, Component)]
pub struct MandelbrotZoom(pub f32);

pub fn mandelbrot_zoom(start: f32, end: f32) -> TweenMandelbrotZoom {
    TweenMandelbrotZoom::new(start, end)
}

#[derive(Component)]
pub struct TweenMandelbrotZoom {
    start: f32,
    end: f32,
}

impl TweenMandelbrotZoom {
    pub fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }
}

impl Interpolator for TweenMandelbrotZoom {
    type Item = MandelbrotZoom;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.0 = self.start.lerp(self.end, value);
    }
}
