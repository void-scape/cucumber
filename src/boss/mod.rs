use bevy::prelude::*;

pub mod gradius;

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(gradius::GradiusPlugin);
    }
}
