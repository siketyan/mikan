use crate::graphics::fonts::{Font, Shinonome};
use crate::graphics::{Canvas, Color, Position};

pub(crate) const FONT_HEIGHT: usize = 16;
pub(crate) const FONT_WIDTH: usize = 8;

pub(crate) trait TextWriter: Canvas {
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

    fn write_chars<C>(&mut self, position: Position, chars: C, color: Color)
    where
        C: IntoIterator<Item = char>,
    {
        chars.into_iter().enumerate().for_each(|(i, c)| {
            self.write_ascii(position + (FONT_WIDTH * i, FONT_HEIGHT).into(), c, color)
        })
    }

    fn write_string(&mut self, position: Position, string: &str, color: Color) -> Position {
        string
            .split('\n')
            .enumerate()
            .fold(position, |mut p, (i, line)| {
                p = (if i > 0 { 0 } else { p.x }, p.y + i * FONT_HEIGHT).into();
                self.write_chars(p, line.chars(), color);
                p + (line.chars().count() * FONT_WIDTH, 0).into()
            })
    }
}

impl<T> TextWriter for T where T: Canvas {}
