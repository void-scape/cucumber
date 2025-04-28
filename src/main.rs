#![allow(clippy::type_complexity)]

use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_optix::camera::MainCamera;
use bevy_optix::pixel_perfect::{CanvasDimensions, Scaling};
use bevy_optix::shake::prelude::*;
use physics::layers::{self, RegisterPhysicsLayer};
use physics::prelude::Gravity;

mod animation;
mod assets;
mod auto_collider;
mod background;
mod bounds;
mod bullet;
mod characters;
mod enemy;
mod fire;
mod health;
mod music;
mod opening;
mod pickups;
mod player;
mod textbox;
mod ui;

pub const WIDTH: f32 = 128.;
pub const HEIGHT: f32 = 256.;

pub const RESOLUTION_SCALE: f32 = 3.;
pub const RES_WIDTH: f32 = WIDTH * 1.25;
pub const RES_HEIGHT: f32 = HEIGHT;

fn main() {
    let mut app = App::new();

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(
                        RES_WIDTH * RESOLUTION_SCALE,
                        RES_HEIGHT * RESOLUTION_SCALE,
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        bevy_enhanced_input::EnhancedInputPlugin,
        bevy_tween::DefaultTweenPlugins,
        bevy_seedling::SeedlingPlugin::default(),
        physics::PhysicsPlugin,
        bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions::new(
            WIDTH as u32,
            HEIGHT as u32,
        )),
        bevy_optix::shake::ScreenShakePlugin,
    ))
    .add_plugins((
        animation::AnimationPlugin,
        music::MusicPlugin,
        assets::AssetPlugin,
        pickups::PickupPlugin,
        characters::CharacterPlugin,
        player::PlayerPlugin,
        enemy::EnemyPlugin,
        textbox::TextboxPlugin,
        background::BackgroundPlugin,
        bullet::BulletPlugin,
        health::HealthPlugin,
        auto_collider::AutoColliderPlugin,
        bounds::ScreenBoundsPlugin,
        ui::UiPlugin,
        opening::OpeningPlugin,
    ))
    .insert_state(GameState::Opening)
    .add_systems(Startup, configure_screen_shake)
    .register_collision_layer::<layers::Player>(32.0)
    .register_collision_layer::<layers::Enemy>(32.0)
    .register_collision_layer::<layers::Wall>(32.0)
    .register_trigger_layer::<physics::layers::Enemy>()
    .register_trigger_layer::<physics::layers::Player>()
    .register_trigger_layer::<physics::layers::Wall>()
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Gravity(Vec2::ZERO))
    .insert_resource(Scaling::Canvas)
    .run();
}

#[cfg(debug_assertions)]
fn close_on_escape(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.read() {
        if e.key_code == KeyCode::Escape && e.state == ButtonState::Pressed {
            writer.write(AppExit::Success);
        }
    }
}

fn configure_screen_shake(mut commands: Commands, main_camera: Single<Entity, With<MainCamera>>) {
    commands
        .entity(*main_camera)
        .insert(Shake::from_trauma_limit(1.));
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Opening,
    Game,
}
