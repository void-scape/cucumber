use crate::assets::{PROJECTILES_COLORED_PATH, SHIPS_PATH};
use crate::bomb::Bombs;
use crate::health::Health;
use crate::player::Player;
use crate::points::{self, Points};
use crate::sprites::CellSize;
use crate::text::TextFlash;
use crate::{DespawnRestart, GameState, sprites};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PointAccumulator(0))
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(OnEnter(GameState::StartGame), ui)
            .add_systems(Update, update_ui)
            .add_systems(FixedUpdate, accumulate_points);
    }
}

fn restart(mut commands: Commands) {
    commands.insert_resource(PointAccumulator(0));
}

#[derive(Component)]
struct LivesText;

#[derive(Component)]
struct BombText;

#[derive(Component)]
struct GamePointText;

fn ui(mut commands: Commands, server: Res<AssetServer>) {
    let mut lives_sprite =
        sprites::sprite_rect(&server, SHIPS_PATH, CellSize::Eight, UVec2::new(1, 5));
    lives_sprite.anchor = Anchor::TopLeft;
    commands
        .spawn((
            DespawnRestart,
            LivesText,
            HIGH_RES_LAYER,
            Text2d::default(),
            TextFont {
                font_size: 32.,
                font: server.load("fonts/gravity.ttf"),
                ..Default::default()
            },
            Transform::from_xyz(
                -crate::WIDTH / 2. * crate::RESOLUTION_SCALE + 12. * crate::RESOLUTION_SCALE,
                crate::HEIGHT / 2. * crate::RESOLUTION_SCALE - 10.,
                500.,
            ),
            Anchor::TopLeft,
        ))
        .with_child((
            lives_sprite,
            Transform::from_xyz(-10. * crate::RESOLUTION_SCALE, -crate::RESOLUTION_SCALE, 0.)
                .with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        ));

    let mut bomb_sprite = sprites::sprite_rect(
        &server,
        PROJECTILES_COLORED_PATH,
        CellSize::Eight,
        UVec2::new(4, 3),
    );
    bomb_sprite.anchor = Anchor::TopLeft;
    commands
        .spawn((
            DespawnRestart,
            BombText,
            HIGH_RES_LAYER,
            Text2d::default(),
            TextFont {
                font_size: 32.,
                font: server.load("fonts/gravity.ttf"),
                ..Default::default()
            },
            Transform::from_xyz(
                -crate::WIDTH / 2. * crate::RESOLUTION_SCALE + 32. * crate::RESOLUTION_SCALE,
                crate::HEIGHT / 2. * crate::RESOLUTION_SCALE - 10.,
                500.,
            ),
            Anchor::TopLeft,
        ))
        .with_child((
            bomb_sprite,
            Transform::from_xyz(-10. * crate::RESOLUTION_SCALE, -crate::RESOLUTION_SCALE, 0.)
                .with_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        ));

    commands.spawn((
        DespawnRestart,
        GamePointText,
        HIGH_RES_LAYER,
        Text2d::default(),
        TextFont {
            font_size: 32.,
            font: server.load("fonts/gravity.ttf"),
            ..Default::default()
        },
        Transform::from_xyz(
            crate::WIDTH / 2. * crate::RESOLUTION_SCALE - 2. * crate::RESOLUTION_SCALE,
            crate::HEIGHT / 2. * crate::RESOLUTION_SCALE - 10.,
            500.,
        ),
        Anchor::TopRight,
    ));
}

#[derive(Resource)]
struct PointAccumulator(usize);

fn accumulate_points(
    mut commands: Commands,
    mut accumulator: ResMut<PointAccumulator>,
    points: Res<Points>,
    text: Single<(Entity, Option<&TextFlash>), With<GamePointText>>,
) {
    let (text, point_text) = text.into_inner();

    let points = points.get();
    if accumulator.0 < points {
        accumulator.0 += 1;
        if point_text.is_none() {
            commands
                .entity(text)
                .insert(TextFlash::new(0.2, Color::WHITE, points::COLOR));
        }
    } else if point_text.is_some() {
        commands.entity(text).remove::<TextFlash>();
    }
}

fn update_ui(
    mut live_text: Single<&mut Text2d, With<LivesText>>,
    mut bomb_text: Single<&mut Text2d, (With<BombText>, Without<LivesText>)>,
    mut point_text: Single<
        &mut Text2d,
        (With<GamePointText>, Without<LivesText>, Without<BombText>),
    >,
    player: Single<Ref<Health>, With<Player>>,
    bombs: Res<Bombs>,
    points: Res<PointAccumulator>,
) {
    if player.is_changed() {
        live_text.0 = format!("{}", (player.current() - 1.).max(0.));
    }

    if bombs.is_changed() {
        bomb_text.0 = format!("{}", bombs.get());
    }

    if points.is_changed() {
        point_text.0 = format!("{}", points.0);
    }
}
