use crate::GameState;
use crate::bullet::{Bullet, PlayerBullet};
use crate::effects::{Explosion, SpawnExplosion};
use crate::pickups::Bomb;
use crate::player::{AliveContext, Player};
use crate::points::PointEvent;
use avian2d::prelude::CollidingEntities;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;

const STARTING_BOMBS: usize = 3;

pub struct BombPlugin;

impl Plugin for BombPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Bombs::new(STARTING_BOMBS))
            .add_systems(OnEnter(GameState::StartGame), insert_bombs)
            .add_systems(Update, collect_bombs)
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

    pub fn get(&self) -> usize {
        self.0
    }
}

fn insert_bombs(mut commands: Commands) {
    commands.insert_resource(Bombs::new(STARTING_BOMBS));
}

#[derive(Debug, InputAction)]
#[input_action(output = bool, consume_input = false)]
pub struct BombAction;

fn bind(trigger: Trigger<Binding<AliveContext>>, mut actions: Query<&mut Actions<AliveContext>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<BombAction>()
        .to((KeyCode::KeyC, GamepadButton::East))
        .with_conditions(JustPress::default());
}

fn detonate(
    _: Trigger<Fired<BombAction>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut bombs: ResMut<Bombs>,
    mut points: EventWriter<PointEvent>,
    mut explosions: EventWriter<SpawnExplosion>,
    bullets: Query<(Entity, &Transform), (With<Bullet>, Without<PlayerBullet>)>,
    player: Single<&Transform, With<Player>>,
) {
    if bombs.0 != 0 {
        bombs.0 -= 1;

        //commands.spawn((
        //    SamplePlayer::new(server.load("audio/sfx/explosion4.wav")),
        //    //PlaybackParams {
        //    //    speed: 0.75,
        //    //    ..Default::default()
        //    //},
        //    PlaybackSettings {
        //        volume: Volume::Linear(0.45),
        //        ..PlaybackSettings::ONCE
        //    },
        //));
        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/note2.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.5),
                ..PlaybackSettings::ONCE
            },
        ));

        let position = player.translation.xy();
        explosions.write(SpawnExplosion {
            position: position + Vec2::new(15., -15.),
            explosion: Explosion::Big,
        });
        explosions.write(SpawnExplosion {
            position: position + Vec2::new(5., 10.),
            explosion: Explosion::Big,
        });
        explosions.write(SpawnExplosion {
            position: position + Vec2::new(-10., -10.),
            explosion: Explosion::Big,
        });

        for (entity, transform) in bullets.iter() {
            commands.entity(entity).despawn();
            points.write(PointEvent {
                position: transform.translation.xy(),
                points: 2,
            });
            explosions.write(SpawnExplosion {
                position: transform.translation.xy(),
                explosion: Explosion::Small,
            });

            //commands.spawn((
            //    Material::Parts,
            //    LinearVelocity(Vec2::NEG_Y * MATERIAL_SPEED),
            //    *transform,
            //));
        }
    }
}

fn collect_bombs(
    mut commands: Commands,
    server: Res<AssetServer>,
    player: Single<&CollidingEntities, With<Player>>,
    pickups: Query<&Bomb>,
    mut bombs: ResMut<Bombs>,
) {
    for entity in player
        .iter()
        .copied()
        .filter(|entity| pickups.get(*entity).is_ok())
    {
        commands.entity(entity).despawn();
        bombs.0 += 1;

        commands.spawn((
            SamplePlayer::new(server.load("audio/sfx/bfxr/bomb.wav")),
            PlaybackSettings {
                volume: Volume::Linear(0.5),
                ..PlaybackSettings::ONCE
            },
        ));
    }
}
