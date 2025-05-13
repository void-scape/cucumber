use crate::stats::Stats;
use crate::{DespawnRestart, GameState, input};
use bevy::prelude::*;
use bevy_enhanced_input::events::Fired;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

pub struct EndPlugin;

impl Plugin for EndPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(restart);
    }
}

#[derive(Component)]
struct EndScreen;

pub fn show_win_screen(mut commands: Commands, server: Res<AssetServer>, stats: Res<Stats>) {
    commands.spawn((EndScreen, DespawnRestart));

    commands.spawn((
        DespawnRestart,
        HIGH_RES_LAYER,
        Text2d("You Won!".into()),
        TextFont {
            font_size: 30.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_xyz(0., 80., 500.),
    ));

    let stats = [
        (40., format!("Time: {:.2}", stats.time.elapsed_secs())),
        (20., format!("Kills: {}", stats.kills)),
        (0., format!("Materials: {}", stats.materials)),
        (-crate::HEIGHT / 2., "[ Restart ]".into()),
    ];

    for (y, text) in stats.into_iter() {
        commands.spawn((
            DespawnRestart,
            HIGH_RES_LAYER,
            Text2d(text),
            TextFont {
                font_size: 20.,
                font: server.load("fonts/joystix.otf"),
                ..Default::default()
            },
            Transform::from_xyz(0., y, 500.),
        ));
    }

    commands.spawn((
        DespawnRestart,
        Sprite {
            rect: Some(Rect::from_center_size(
                Vec2::ZERO,
                Vec2::new(crate::WIDTH, crate::HEIGHT),
            )),
            color: Color::linear_rgba(0., 0., 0., 0.9),
            ..Default::default()
        },
        Transform::from_xyz(0., 0., 499.),
    ));
}

pub fn show_loose_screen(mut commands: Commands, server: Res<AssetServer>, stats: Res<Stats>) {
    commands.spawn((EndScreen, DespawnRestart));

    commands.spawn((
        DespawnRestart,
        HIGH_RES_LAYER,
        Text2d("You Died...".into()),
        TextFont {
            font_size: 30.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_xyz(0., 80., 500.),
    ));

    let stats = [
        (40., format!("Time: {:.2}", stats.time.elapsed_secs())),
        (20., format!("Kills: {}", stats.kills)),
        (0., format!("Materials: {}", stats.materials)),
        (-crate::HEIGHT / 2., "[ Restart ]".into()),
    ];

    for (y, text) in stats.into_iter() {
        commands.spawn((
            DespawnRestart,
            HIGH_RES_LAYER,
            Text2d(text),
            TextFont {
                font_size: 20.,
                font: server.load("fonts/joystix.otf"),
                ..Default::default()
            },
            Transform::from_xyz(0., y, 500.),
        ));
    }

    commands.spawn((
        DespawnRestart,
        Sprite {
            rect: Some(Rect::from_center_size(
                Vec2::ZERO,
                Vec2::new(crate::WIDTH, crate::HEIGHT),
            )),
            color: Color::linear_rgba(0., 0., 0., 0.9),
            ..Default::default()
        },
        Transform::from_xyz(0., 0., 499.),
    ));
}

fn restart(
    _: Trigger<Fired<input::Interact>>,
    _enable: Single<&EndScreen>,
    mut commands: Commands,
) {
    commands.set_state(GameState::Restart);
}
