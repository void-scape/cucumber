use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<MenuContext>()
            .add_systems(Startup, startup)
            .add_observer(menu_binding);
    }
}

#[derive(InputContext)]
pub struct MenuContext;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Interact;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Left;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Right;

fn startup(mut commands: Commands) {
    commands.spawn(Actions::<MenuContext>::default());
}

fn menu_binding(
    trigger: Trigger<Binding<MenuContext>>,
    mut actions: Query<&mut Actions<MenuContext>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Interact>()
        .to((KeyCode::Space, KeyCode::Enter, GamepadButton::South))
        .with_conditions(JustPress::default());
    actions
        .bind::<Left>()
        .to((KeyCode::KeyA, GamepadButton::DPadLeft))
        .with_conditions(JustPress::default());
    actions
        .bind::<Right>()
        .to((KeyCode::KeyD, GamepadButton::DPadRight))
        .with_conditions(JustPress::default());
}
