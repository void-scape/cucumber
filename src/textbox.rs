use crate::input;
use bevy::prelude::*;
use bevy_enhanced_input::events::Fired;

pub struct TextboxPlugin;

impl Plugin for TextboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_textbox::TextboxPlugin)
            .add_observer(receive_textbox_input);
    }
}

fn receive_textbox_input(
    _: Trigger<Fired<input::Interact>>,
    mut writer: EventWriter<bevy_pretty_text::prelude::Input>,
) {
    writer.write(bevy_pretty_text::prelude::Input::Interact);
}
