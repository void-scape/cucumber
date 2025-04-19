use crate::movement::Velocity;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands,
             mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<ColorMaterial>>| {
                let mut actions = Actions::<AliveContext>::default();
                actions.bind::<MoveAction>().to((
                    Cardinal::wasd_keys(),
                    Cardinal::arrow_keys(),
                    GamepadStick::Left,
                ));

                commands.spawn((
                    Player,
                    Transform::default().with_translation(Vec3::new(200.0, 200.0, 1.0)),
                    actions,
                    Mesh2d(meshes.add(Circle::new(20.0))),
                    MeshMaterial2d(materials.add(Color::WHITE)),
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
    velocity.0 = trigger.value * 200.;
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
