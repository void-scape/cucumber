use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_pixel_gfx::pixel_perfect::CanvasDimensions;
use physics::prelude::Gravity;

mod background;
mod enemy;
mod player;
mod textbox;

pub const WIDTH: f32 = 256.;
pub const HEIGHT: f32 = 256.;
pub const RESOLUTION_SCALE: f32 = 3.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(
                            WIDTH * RESOLUTION_SCALE,
                            HEIGHT * RESOLUTION_SCALE,
                        ),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            bevy_enhanced_input::EnhancedInputPlugin,
            bevy_tween::DefaultTweenPlugins,
            player::PlayerPlugin,
            enemy::EnemyPlugin,
            textbox::TextboxPlugin,
            background::BackgroundPlugin,
            bevy_pixel_gfx::pixel_perfect::PixelPerfectPlugin(CanvasDimensions::new(
                WIDTH as u32,
                HEIGHT as u32,
            )),
            bevy_pixel_gfx::screen_shake::ScreenShakePlugin,
            physics::PhysicsPlugin,
        ))
        .add_systems(Update, close_on_escape)
        .insert_resource(Gravity(Vec2::ZERO))
        .run();
}

fn close_on_escape(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.read() {
        if e.key_code == KeyCode::Escape && e.state == ButtonState::Pressed {
            writer.send(AppExit::Success);
        }
    }
}
