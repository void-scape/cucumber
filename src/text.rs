use crate::color::HexColor;
use crate::tween::DespawnTweenFinish;
use avian2d::prelude::{LinearVelocity, RigidBody};
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_tween::prelude::*;
use bevy_tween::tween::apply_component_tween_system;
use physics::linear_velocity;

pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_text_flash)
            .add_tween_systems(apply_component_tween_system::<TextAlpha>);
    }
}

#[derive(Component)]
pub struct TextAlpha {
    start: f32,
    end: f32,
}

pub fn text_alpha(start: f32, end: f32) -> TextAlpha {
    TextAlpha { start, end }
}

impl Interpolator for TextAlpha {
    type Item = TextColor;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.0.set_alpha(self.start.lerp(self.end, value));
    }
}

#[derive(Component)]
#[component(on_remove = Self::reset)]
pub struct TextFlash {
    pub interval: f32,
    pub normal: Color,
    pub flash: Color,
}

impl TextFlash {
    pub fn new(interval: f32, normal: impl Into<Color>, flash: impl Into<Color>) -> Self {
        Self {
            interval,
            normal: normal.into(),
            flash: flash.into(),
        }
    }
}

impl TextFlash {
    fn reset(mut world: DeferredWorld, ctx: HookContext) {
        if let Some(flash) = world.get::<TextFlash>(ctx.entity) {
            let normal = flash.normal;
            world
                .commands()
                .entity(ctx.entity)
                .try_insert(TextColor(normal));
        }
    }
}

#[derive(Component)]
struct FlashTimer(Timer);

fn update_text_flash(
    mut commands: Commands,
    mut text: Query<(
        Entity,
        &TextFlash,
        &mut TextColor,
        Option<&mut FlashTimer>,
        Option<&TextAlpha>,
    )>,
    time: Res<Time>,
) {
    for (entity, text, mut color, timer, alpha) in text.iter_mut() {
        let Some(mut timer) = timer else {
            commands
                .entity(entity)
                .insert(FlashTimer(Timer::from_seconds(
                    text.interval,
                    TimerMode::Repeating,
                )));
            continue;
        };

        timer.0.tick(time.delta());
        if timer.0.finished() {
            if color.to_srgba().with_alpha(1.) == text.normal.to_srgba().with_alpha(1.) {
                if alpha.is_some() {
                    let a = color.0.alpha();
                    color.0 = text.flash.with_alpha(a);
                } else {
                    color.0 = text.flash;
                }
            } else {
                if alpha.is_some() {
                    let a = color.0.alpha();
                    color.0 = text.normal.with_alpha(a);
                } else {
                    color.0 = text.normal;
                }
            }
        }
    }
}

pub fn flash_text(
    commands: &mut Commands,
    server: &AssetServer,
    text: impl Into<String>,
    size: f32,
    position: Vec3,
    color: impl Into<Color>,
) {
    let text = commands
        .spawn((
            HIGH_RES_LAYER,
            Text2d::new(text.into()),
            TextFont {
                font: server.load("fonts/gravity.ttf"),
                font_size: size,
                ..Default::default()
            },
            Transform::from_translation(position),
            TextFlash::new(0.1, Color::WHITE, color),
            RigidBody::Kinematic,
            LinearVelocity::default(),
        ))
        .id();
    commands
        .entity(text)
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(0.75),
            EaseKind::QuarticIn,
            text.into_target()
                .with(linear_velocity(Vec2::Y * 20., Vec2::ZERO)),
        )
        .insert(DespawnTweenFinish);
    commands
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(0.75),
            EaseKind::QuarticIn,
            text.into_target().with(text_alpha(1., 0.)),
        )
        .insert(DespawnTweenFinish);
}
