use crate::graphics::fonts::{Font, Shinonome};
use crate::graphics::Position;
use crate::{Canvas, Color};

const FONT_HEIGHT: usize = 16;
const FONT_WIDTH: usize = 8;

pub(crate) trait TextWriter<'a>: Canvas<'a> {
    fn write_glyph(&mut self, position: Position, glyph: &[u8], color: Color) {
        #[allow(clippy::needless_range_loop)]
        for y in 0..FONT_HEIGHT {
            for x in 0..FONT_WIDTH {
                if (glyph[y] << x) & 0x80 != 0 {
                    if let Some(mut p) = self.at(position + (x, y).into()) {
                        p.write(color);
                    }
                }
            }
        }
    }

    fn write_ascii(&mut self, position: Position, c: char, color: Color) {
        if let Some(glyph) = Shinonome::glyph(c) {
            self.write_glyph(position, glyph, color)
        }
    }
}

impl<'a, T> TextWriter<'a> for T where T: Canvas<'a> {}
