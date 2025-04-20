use bevy::prelude::*;
use bevy_enhanced_input::events::Started;
use bevy_enhanced_input::prelude::{Actions, InputAction, InputContext, InputContextAppExt};
use bevy_pretty_text::type_writer::input;
use bevy_seedling::prelude::*;
use bevy_sequence::prelude::*;

//use bevy::sprite::Anchor;
//use bevy_pixel_gfx::pixel_perfect::HIGH_RES_LAYER;
//use bevy_pretty_text::prelude::*;
//use bevy_sequence::prelude::*;
//use bevy_textbox::*;

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

//fn test(mut commands: Commands, asset_server: Res<AssetServer>) {
//    let entity = commands
//        .spawn((
//            HIGH_RES_LAYER,
//            TextBox,
//            Sprite {
//                image: asset_server.load("textbox.png"),
//                anchor: Anchor::TopLeft,
//                ..Default::default()
//            },
//            SfxRate::new(
//                0.1,
//                SamplePlayer::new(asset_server.load("audio/sfx/snd_txtral.wav")),
//            ),
//        ))
//        .with_child((
//            Sprite {
//                image: asset_server.load("continue.png"),
//                anchor: Anchor::TopLeft,
//                ..Default::default()
//            },
//            Transform::from_translation(Vec3::default().with_z(100.)),
//            Continue,
//            Visibility::Hidden,
//        ))
//        .id();
//
//    let frag = (s!("`Hello|green`[0.5], `World`[wave]!"), "My name is Nic.")
//        .always()
//        .once()
//        .on_end(move |mut commands: Commands| commands.entity(entity).despawn_recursive());
//    spawn_root_with(frag, &mut commands, TextBoxEntity::new(entity));
//}
