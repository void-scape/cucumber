use crate::RESOLUTION_SCALE;
use crate::enemy::EnemyDeathEvent;
use bevy::color::palettes::css::{WHITE, YELLOW};
use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_seedling::prelude::*;
use bevy_tween::prelude::*;
use bevy_tween::tween::apply_component_tween_system;

pub struct PointPlugin;

impl Plugin for PointPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointEvent>()
            .add_systems(
                PostUpdate,
                (score_enemy_death, point_effects, update_point_text).chain(),
            )
            .add_tween_systems(apply_component_tween_system::<TextColorTween>);
    }
}

#[derive(Event)]
pub struct PointEvent {
    pub points: usize,
    pub position: Vec2,
}

fn score_enemy_death(
    mut reader: EventReader<EnemyDeathEvent>,
    mut writer: EventWriter<PointEvent>,
) {
    for event in reader.read() {
        writer.write(PointEvent {
            points: 20,
            position: event.position,
        });
    }
}

#[derive(Component)]
struct PointText {
    timer: Timer,
    max: usize,
    count: usize,
}

impl Default for PointText {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            max: 8,
            count: 0,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Reflect)]
struct TextColorTween {
    start: Color,
    end: Color,
}

impl Interpolator for TextColorTween {
    type Item = TextColor;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.0 = self.start.mix(&self.end, value)
    }
}

fn text_color(start: impl Into<Color>, end: impl Into<Color>) -> TextColorTween {
    TextColorTween {
        start: start.into(),
        end: end.into(),
    }
}

fn point_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<PointEvent>,
) {
    if !reader.is_empty() {
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/bfxr/Random50.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.35),
                ..PlaybackSettings::ONCE
            },
        ));
    }

    for event in reader.read() {
        commands.spawn((
            HIGH_RES_LAYER,
            Text2d::new(format!("+{}", event.points)),
            TextFont {
                font: server.load("fonts/gravity.ttf"),
                font_size: 32.,
                ..Default::default()
            },
            Transform::from_translation((event.position * RESOLUTION_SCALE).extend(500.)),
            PointText::default(),
        ));
    }
}

fn update_point_text(
    mut commands: Commands,
    mut text: Query<(Entity, &mut PointText, &mut Transform, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut text, mut transform, mut color) in text.iter_mut() {
        transform.translation.y += 20. * time.delta_secs();
        text.timer.tick(time.delta());
        if text.timer.finished() {
            text.count += 1;
            if text.count >= text.max {
                commands.entity(entity).despawn();
            } else {
                if color.to_srgba() == WHITE {
                    color.0 = YELLOW.into();
                } else {
                    color.0 = WHITE.into();
                }
            }
        }
    }
}
