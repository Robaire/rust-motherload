extern crate freetype;
extern crate sdl2;

use super::gl_util;
use std::collections::HashMap;

pub struct Glyph {
    buffer_id: u32, // ID of the texture buffer containing this character
    size: (i32, i32),
    bearing: (i32, i32),
    advance: i32,
}

pub fn from_file(
    library: &mut freetype::library::Library,
    video_subsystem: &sdl2::VideoSubsystem,
    file: &str,
) -> Result<HashMap<u8, Glyph>, String> {
    let face = match library.new_face(file, 0) {
        Ok(face) => face,
        Err(error) => return Err(error.to_string()),
    };

    let dpi = match video_subsystem.display_dpi(0) {
        Ok(dpi) => dpi,
        Err(_) => (200.0, 200.0, 200.0),
    };

    // The size of the font in points
    let points = 32;
    match face.set_char_size(0, points * 64, dpi.0 as u32, dpi.1 as u32) {
        Ok(_) => (),
        Err(error) => return Err(error.to_string()),
    };

    let mut glyph_map: HashMap<u8, Glyph> = HashMap::new();

    // Extract the glyph bitmap for every ASCII glyph
    for i in 0..128 {
        // Attempt to load the glyph
        match face.load_char(i, freetype::face::LoadFlag::RENDER) {
            Ok(_) => (),
            Err(_) => continue,
        }

        let glyph = face.glyph();

        // TODO: Generate the buffer id
        let buffer_id = 0;

        // TODO
        // Copy bitmap data to the buffer

        // Add the glyph to the map
        glyph_map.insert(i as u8, {
            Glyph {
                buffer_id,
                size: (glyph.bitmap().width(), glyph.bitmap().rows()),
                bearing: (glyph.bitmap_left(), glyph.bitmap_top()),
                advance: glyph.advance().x as i32,
            }
        });
    }

    return Ok(glyph_map);
}
