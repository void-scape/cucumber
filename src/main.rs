#![allow(clippy::type_complexity)]

use avian2d::prelude::{Gravity, PhysicsDebugPlugin, PhysicsLayer};
use bevy::app::FixedMainScheduleOrder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_optix::camera::MainCamera;
use bevy_optix::pixel_perfect::{CanvasDimensions, Scaling};
use bevy_optix::shake::prelude::*;

mod animation;
mod assets;
mod asteroids;
mod auto_collider;
mod background;
mod bounds;
mod bullet;
mod characters;
mod enemy;
mod fire;
mod health;
mod miniboss;
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
        bevy_seedling::SeedlingPlugin {
            default_pool_size: Some(32),
            ..Default::default()
        },
        // the average object (bullet) is 8 ppx.
        avian2d::PhysicsPlugins::new(Avian).with_length_unit(8.),
        //PhysicsDebugPlugin::new(Avian),
        bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions::new(
            WIDTH as u32,
            HEIGHT as u32,
        )),
        bevy_optix::shake::ScreenShakePlugin,
        bevy_optix::debug::DebugPlugin,
        physics::PhysicsPlugin,
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
    .add_plugins((miniboss::MinibossPlugin, asteroids::AsteroidPlugin))
    .init_schedule(Avian)
    .insert_resource(Gravity(Vec2::ZERO))
    .init_state::<GameState>()
    .add_systems(Startup, set_state.run_if(in_state(GameState::Startup)))
    .add_systems(Startup, configure_screen_shake)
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(Scaling::Canvas);

    // the defalt schedule for Avian is `FixedPostUpdate`, but I wanted something easier to type,
    // so it is set to `Avian`
    app.world_mut()
        .resource_mut::<FixedMainScheduleOrder>()
        .insert_after(FixedPostUpdate, Avian);

    app.run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Avian;

// this many layers is probably not necessary
#[derive(Default, Clone, Copy, PhysicsLayer)]
pub enum Layer {
    #[default]
    Bounds,
    Bullet,
    Player,
    Enemy,
}

fn set_state(mut commands: Commands) {
    commands.set_state(GameState::Game);
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
    Startup,
    Opening,
    Game,
}
