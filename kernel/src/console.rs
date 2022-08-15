use core::fmt::Write;

use crate::graphics::text::{FONT_HEIGHT, FONT_WIDTH};
use crate::graphics::{Color, Position};
use crate::{Colors, TextWriter};

const DEFAULT_ROWS: usize = 25;
const DEFAULT_COLUMNS: usize = 80;

pub(crate) struct Console<'a, W> {
    writer: &'a mut W,
    position: Position,
    cursor: (usize, usize),
    color: Color,
    rows: usize,
    columns: usize,
}

impl<'a, W> Console<'a, W> {
    pub(crate) fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            position: Position::zero(),
            cursor: (0, 0),
            color: Colors::black(),
            rows: DEFAULT_ROWS,
            columns: DEFAULT_COLUMNS,
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

    pub(crate) fn position(&self) -> Position {
        let (x, y) = self.cursor;
        self.position + (x * FONT_WIDTH, y * FONT_HEIGHT).into()
    }

    pub(crate) fn next(&mut self) {
        let (x, y) = self.cursor;
        if x >= self.columns - 1 {
            self.new_line()
        } else {
            self.cursor = (x + 1, y)
        }
    }

    pub(crate) fn new_line(&mut self) {
        let (_, y) = self.cursor;
        self.cursor = (0, y + 1);
    }
}

impl<'a, W> Write for Console<'a, W>
where
    W: TextWriter<'a>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().for_each(|c| {
            if c == '\n' {
                return self.new_line();
            }

            self.writer.write_ascii(self.position(), c, self.color);
            self.next()
        });

        Ok(())
    }
}
