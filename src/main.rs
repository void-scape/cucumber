use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::prelude::*;
//use bevy_pixel_gfx::pixel_perfect::CanvasDimensions;
use physics::prelude::Gravity;

mod invaders;
mod mandelbrot;
mod player;
mod textbox;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            bevy_enhanced_input::EnhancedInputPlugin,
            bevy_tween::DefaultTweenPlugins,
            player::PlayerPlugin,
            textbox::TextboxPlugin,
            invaders::InvadersSpritePlugin,
            mandelbrot::MandelbrotPlugin,
            //bevy_pixel_gfx::PixelGfxPlugin(CanvasDimensions::new(256, 256)),
            bevy_pixel_gfx::screen_shake::ScreenShakePlugin,
            physics::PhysicsPlugin,
        ))
        .add_systems(Startup, camera)
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

fn camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
