use bevy::prelude::*;
use bevy_enhanced_input::events::Started;
use bevy_enhanced_input::prelude::{Actions, InputAction, InputContext, InputContextAppExt};
use bevy_pretty_text::type_writer::input;

pub struct TextboxPlugin;

impl Plugin for TextboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_textbox::TextboxPlugin)
            .add_systems(Startup, init_textbox_input)
            .add_observer(receive_textbox_input)
            .add_input_context::<TextboxContext>();
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct TextboxInput;

#[derive(InputContext)]
struct TextboxContext;

fn init_textbox_input(mut commands: Commands) {
    let mut actions = Actions::<TextboxContext>::default();
    actions
        .bind::<TextboxInput>()
        .to((KeyCode::Space, KeyCode::Enter, GamepadButton::South));
    commands.spawn(actions);
}

fn receive_textbox_input(_: Trigger<Started<TextboxInput>>, mut writer: EventWriter<input::Input>) {
    writer.send(input::Input::Interact);
}
