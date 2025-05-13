use crate::asteroids::MATERIAL_SPEED;
use crate::bullet::Bullet;
use crate::pickups::Material;
use crate::player::AliveContext;
use crate::{DespawnRestart, GameState};
use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_enhanced_input::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

const STARTING_BOMBS: usize = 3;

pub struct BombPlugin;

impl Plugin for BombPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::StartGame), insert_bombs)
            .add_systems(Update, update_text.run_if(in_state(GameState::Game)))
            .add_observer(bind)
            .add_observer(detonate);
    }
}

#[derive(Resource)]
pub struct Bombs(usize);

impl Bombs {
    pub fn new(count: usize) -> Self {
        Self(count)
    }
}

#[derive(Component)]
struct BombDisplay;

fn insert_bombs(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Bombs::new(STARTING_BOMBS));
    commands.spawn((
        BombDisplay,
        DespawnRestart,
        HIGH_RES_LAYER,
        Text2d::default(),
        TextFont {
            font_size: 20.,
            font: server.load("fonts/joystix.otf"),
            ..Default::default()
        },
        Transform::from_xyz(
            crate::WIDTH / 2. * crate::RESOLUTION_SCALE,
            -crate::HEIGHT / 2. * crate::RESOLUTION_SCALE,
            500.,
        ),
        Anchor::BottomRight,
    ));
}

fn update_text(bombs: Option<Res<Bombs>>, mut text: Single<&mut Text2d, With<BombDisplay>>) {
    if let Some(bombs) = bombs {
        if bombs.is_changed() {
            text.0 = format!("Bombs: {}", bombs.0);
        }
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct BombAction;

fn bind(trigger: Trigger<Binding<AliveContext>>, mut actions: Query<&mut Actions<AliveContext>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<BombAction>()
        .to((KeyCode::KeyC, GamepadButton::North))
        .with_conditions(JustPress::default());
}

fn detonate(
    _: Trigger<Fired<BombAction>>,
    mut commands: Commands,
    mut bombs: ResMut<Bombs>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
) {
    if bombs.0 != 0 {
        bombs.0 -= 1;
        for (entity, transform) in bullets.iter() {
            commands.entity(entity).despawn();
            commands.spawn((
                Material::Parts,
                LinearVelocity(Vec2::NEG_Y * MATERIAL_SPEED),
                *transform,
            ));
        }
    }
}
