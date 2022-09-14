use core::fmt::Write;

use crate::graphics::text::{FONT_HEIGHT, FONT_WIDTH};
use crate::graphics::{Color, Position};
use crate::{Canvas, Colors, Region, TextWriter};

const ROWS: usize = 25;
const COLUMNS: usize = 80;
const WIDTH: usize = COLUMNS * FONT_WIDTH;
const HEIGHT: usize = ROWS * FONT_HEIGHT;

pub(crate) struct Console<W>
where
    W: 'static,
{
    writer: &'static mut W,
    position: Position,
    cursor: (usize, usize),
    background: Color,
    foreground: Color,
    buffer: [[char; COLUMNS + 1]; ROWS],
}

impl<W> Console<W>
where
    W: Canvas,
{
    pub(crate) fn new(writer: &'static mut W) -> Self {
        Self {
            writer,
            position: Position::zero(),
            cursor: (0, 0),
            background: Colors::white(),
            foreground: Colors::black(),
            buffer: [['\0'; COLUMNS + 1]; ROWS],
        }
    }

    pub(crate) fn with_position<P>(mut self, position: P) -> Self
    where
        P: Into<Position>,
    {
        self.position = position.into();
        self
    }

    pub(crate) fn with_background<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.background = color.into();
        self
    }

    pub(crate) fn with_foreground<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.foreground = color.into();
        self
    }

    #[inline]
    pub(crate) fn region(&self) -> Region {
        Region::new(self.position, WIDTH, HEIGHT)
    }

    #[inline]
    pub(crate) fn position(&self) -> Position {
        let (x, y) = self.cursor;
        self.position + (x * FONT_WIDTH, y * FONT_HEIGHT).into()
    }

    pub(crate) fn next(&mut self) {
        let (x, y) = self.cursor;
        if x >= COLUMNS - 1 {
            self.new_line()
        } else {
            self.cursor = (x + 1, y)
        }
    }

    pub(crate) fn new_line(&mut self) {
        let (_, y) = self.cursor;
        if y < ROWS - 1 {
            self.cursor = (0, y + 1);
        } else {
            self.cursor = (0, ROWS - 1);
            self.writer.fill_in(self.region(), self.background);
            self.buffer.copy_within(1.., 0);
            self.buffer.last_mut().unwrap().fill('\0');
            self.buffer.iter().enumerate().for_each(|(i, line)| {
                self.writer.write_chars(
                    self.position + (0, i * FONT_HEIGHT).into(),
                    *line,
                    self.foreground,
                )
            })
        }
    }
}

impl<W> Write for Console<W>
where
    W: TextWriter,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().for_each(|c| {
            let (x, y) = self.cursor;
            if c == '\n' {
                return self.new_line();
            }

            self.writer.write_ascii(self.position(), c, self.foreground);
            self.buffer[y][x] = c;
            self.next()
        });

        Ok(())
    }
}
