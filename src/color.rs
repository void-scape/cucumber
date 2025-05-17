use bevy::prelude::*;

pub struct HexColor(pub u32);

impl Into<Color> for HexColor {
    fn into(self) -> Color {
        Color::srgb_u8(
            (self.0 >> 16) as u8 & 0xFF,
            (self.0 >> 8) as u8 & 0xFF,
            self.0 as u8,
        )
    }
}
