use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_pixel_gfx::pixel_perfect::CanvasDimensions;

mod invaders;
mod mandelbrot;
mod movement;
mod player;
mod textbox;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1080., 1080.),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            movement::MovementPlugin,
            bevy_enhanced_input::EnhancedInputPlugin,
            player::PlayerPlugin,
            textbox::TextboxPlugin,
            invaders::InvadersSpritePlugin,
            mandelbrot::MandelbrotPlugin,
            bevy_pixel_gfx::pixel_perfect::PixelPerfectPlugin(CanvasDimensions::new(256, 256)),
            bevy_pixel_gfx::screen_shake::ScreenShakePlugin,
        ))
        .add_systems(Update, close_on_escape)
        .run();
}

fn close_on_escape(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.read() {
        if e.key_code == KeyCode::Escape && e.state == ButtonState::Pressed {
            writer.send(AppExit::Success);
        }
    }
}
