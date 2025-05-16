#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![windows_subsystem = "windows"]

use avian2d::prelude::{Gravity, PhysicsLayer};
use bevy::app::FixedMainScheduleOrder;
use bevy::core_pipeline::bloom::Bloom;
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
mod bomb;
mod boss;
mod bounds;
mod bullet;
mod characters;
mod effects;
mod end;
mod enemy;
mod fire;
mod health;
mod input;
mod miniboss;
mod minions;
mod music;
mod opening;
mod particles;
mod pickups;
mod player;
mod points;
mod sampler;
mod selection;
mod stats;
mod textbox;
mod tween;
mod ui;

pub const WIDTH: f32 = 128.;
pub const HEIGHT: f32 = 192.;

pub const RESOLUTION_SCALE: f32 = 4.;
pub const RES_WIDTH: f32 = WIDTH;
pub const RES_HEIGHT: f32 = HEIGHT;

pub const METER: f32 = 8.;

pub const SKIP_WAVES: bool = false;

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
        bevy_tween::DefaultTweenPlugins,
        bevy_seedling::SeedlingPlugin {
            ..Default::default()
        },
        bevy_enhanced_input::EnhancedInputPlugin,
        // the average object (bullet) is 8 ppx.
        avian2d::PhysicsPlugins::new(Avian).with_length_unit(METER),
        //avian2d::debug_render::PhysicsDebugPlugin::new(Avian),
        bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            pixel_scale: RESOLUTION_SCALE,
        }),
        bevy_optix::shake::ScreenShakePlugin,
        bevy_optix::debug::DebugPlugin,
        physics::PhysicsPlugin,
        bevy_enoki::EnokiPlugin,
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
    .add_plugins((
        //miniboss::MinibossPlugin,
        asteroids::AsteroidPlugin,
        minions::MinionPlugin,
        tween::TweenPlugin,
        selection::SelectionPlugin,
        stats::StatPlugin,
        end::EndPlugin,
        input::InputPlugin,
        boss::BossPlugin,
        bomb::BombPlugin,
        points::PointPlugin,
        effects::EffectsPlugin,
        particles::ParticlePlugin,
    ))
    .init_schedule(Avian)
    .insert_resource(Gravity(Vec2::ZERO))
    .init_state::<GameState>()
    .add_systems(Startup, finish_startup.run_if(in_state(GameState::Startup)))
    .add_systems(
        Update,
        despawn_on_restart.run_if(in_state(GameState::Restart)),
    )
    .add_systems(
        Update,
        (
            enter_game.run_if(in_state(GameState::StartGame)),
            enter_start_game.run_if(in_state(GameState::Restart)),
        ),
    )
    .add_systems(Startup, configure_camera)
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
#[derive(Default, Clone, Copy, PartialEq, Eq, PhysicsLayer)]
pub enum Layer {
    #[default]
    Default,
    Bounds,
    Bullet,
    Player,
    Enemy,
    Debris,
    Collectable,
    Miners,
}

fn finish_startup(mut commands: Commands) {
    #[cfg(not(debug_assertions))]
    commands.set_state(GameState::Opening);
    #[cfg(debug_assertions)]
    commands.set_state(GameState::StartGame);
}

fn enter_game(mut commands: Commands) {
    commands.set_state(GameState::Game)
}

fn enter_start_game(mut commands: Commands) {
    commands.set_state(GameState::StartGame)
}

#[cfg(debug_assertions)]
fn close_on_escape(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.read() {
        if e.key_code == KeyCode::Escape && e.state == ButtonState::Pressed {
            writer.write(AppExit::Success);
        }
    }
}

fn configure_camera(mut commands: Commands, main_camera: Single<Entity, With<MainCamera>>) {
    commands.entity(*main_camera).insert((
        Shake::from_trauma_limit(0.7),
        // Bloom::ANAMORPHIC,
    ));
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Startup,
    Opening,
    StartGame,
    Restart,
    Game,
    Selection,
}

#[derive(Default, Component)]
pub struct DespawnRestart;

fn despawn_on_restart(
    mut commands: Commands,
    entities: Query<Entity, (With<DespawnRestart>, Without<Children>)>,
    parents: Query<Entity, (With<DespawnRestart>, With<Children>)>,
) {
    for entity in entities.iter() {
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .despawn();
    }

    for entity in parents.iter() {
        commands.entity(entity).despawn();
    }
}
