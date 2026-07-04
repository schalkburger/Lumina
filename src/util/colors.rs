use freya::prelude::Color;

pub const RED_GRAY: Color = Color::new(0xFF3F2226); // #3F2226
pub const DARKISH_GRAY: Color = Color::new(0xFF242428); // #242428
pub const DARKISH_BLUE: Color = Color::new(0xFF121927); // #121927
pub const DARKER_GRAY: Color = Color::new(0xFF252b32); // #252b32
pub const GRAY: Color = Color::new(0xFF1E1F23); // #1E1F23
pub const LIGHT_GRAY: Color = Color::new(0xFF37373C); // #37373C
pub const SUPERLIGHT_GRAY: Color = Color::new(0xFFB4B4B4); // #B4B4B4
pub const MUTED_GRAY: Color = Color::new(0xFF6B6B70); // #6B6B70
pub const TRANSPARENT_GRAY: Color = Color::new(0x56222222); // #222222 with 34% opacity
pub const GREEN: Color = Color::new(0xFF01863B); // #01863B
pub const TRANSPARENT: Color = Color::new(0x00000000); // #000000 with 0% opacity

pub fn parse_hex(hex: &str) -> Option<Color> {
  let s = hex.trim().trim_start_matches('#');

  match s.len() {
    6 => {
      let rgb = u32::from_str_radix(s, 16).ok()?;
      Some(Color::new(0xFF000000 | rgb))
    }
    8 => {
      let argb = u32::from_str_radix(s, 16).ok()?;
      Some(Color::new(argb))
    }
    _ => None,
  }
}

pub fn to_argb_hex(c: Color) -> String {
  format!(
    "#{:02X}{:02X}{:02X}{:02X}",
    c.a() as u32,
    c.r() as u32,
    c.g() as u32,
    c.b() as u32,
  )
}
