use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use physics::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands, server: Res<AssetServer>| {
                let mut actions = Actions::<AliveContext>::default();
                actions.bind::<MoveAction>().to((
                    Cardinal::wasd_keys(),
                    Cardinal::arrow_keys(),
                    Cardinal::dpad_buttons(),
                    GamepadStick::Left.with_modifiers_each(
                        DeadZone::new(DeadZoneKind::Radial).with_lower_threshold(0.15),
                    ),
                ));

                commands.spawn((
                    Player,
                    actions,
                    Sprite {
                        image: server.load("invaders_sprites.png"),
                        rect: Some(Rect::from_corners(Vec2::ZERO, Vec2::ONE * 8.)),
                        ..Default::default()
                    },
                ));
            },
        )
        .add_input_context::<AliveContext>()
        .add_observer(apply_movement)
        .add_observer(stop_movement);
    }
}

#[derive(Component)]
#[require(Transform, Velocity)]
pub struct Player;

fn apply_movement(
    trigger: Trigger<Fired<MoveAction>>,
    player: Single<&mut Velocity, With<Player>>,
) {
    let mut velocity = player.into_inner();
    velocity.0 = trigger.value.normalize_or_zero() * 200.;
}

fn stop_movement(_: Trigger<Completed<MoveAction>>, player: Single<&mut Velocity, With<Player>>) {
    let mut velocity = player.into_inner();
    velocity.0 = Vec2::default();
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct MoveAction;

#[derive(InputContext)]
struct AliveContext;
