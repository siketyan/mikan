use crate::graphics::Position;
use crate::{Canvas, Color};

const FONT_HEIGHT: usize = 16;
const FONT_WIDTH: usize = 8;

#[rustfmt::skip]
const FONT_CHAR_A: &[u8; FONT_HEIGHT] = &[
    0b00000000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00100100,
    0b00100100,
    0b00100100,
    0b00100100,
    0b01111110,
    0b01000010,
    0b01000010,
    0b01000010,
    0b11100111,
    0b00000000,
    0b00000000,
];

pub(crate) trait TextWriter<'a>: Canvas<'a> {
    fn write_ascii(&mut self, position: Position, c: char, color: Color) {
        if c != 'A' {
            return;
        }

        #[allow(clippy::needless_range_loop)]
        for y in 0..FONT_HEIGHT {
            for x in 0..FONT_WIDTH {
                if (FONT_CHAR_A[y] << x) & 0x80 != 0 {
                    if let Some(mut p) = self.at(position + (x, y).into()) {
                        p.write(color);
                    }
                }
            }
        }
    }
}

impl<'a, T> TextWriter<'a> for T where T: Canvas<'a> {}
