use core::fmt::Write;

use crate::graphics::fonts::{Font, Shinonome};
use crate::graphics::{Canvas, Color, Colors, Position};

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

impl<'a, T> TextWriter<'a> for T where T: Canvas<'a> {}

pub(crate) struct BufTextWriter<'a, W> {
    writer: &'a mut W,
    position: Position,
    color: Color,
}

impl<'a, W> BufTextWriter<'a, W> {
    pub(crate) fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            position: Position::zero(),
            color: Colors::black(),
        }
    }

    pub(crate) fn with_position<P>(mut self, position: P) -> Self
    where
        P: Into<Position>,
    {
        self.position = position.into();
        self
    }

    pub(crate) fn with_color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.color = color.into();
        self
    }
}

impl<'a, W> Write for BufTextWriter<'a, W>
where
    W: TextWriter<'a>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.position = self.writer.write_string(self.position, s, self.color);
        Ok(())
    }
}
