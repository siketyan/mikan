use core::fmt::Write;

use crate::graphics::{Color, Position};
use crate::{Colors, TextWriter};

pub(crate) struct Console<'a, W> {
    writer: &'a mut W,
    position: Position,
    color: Color,
}

impl<'a, W> Console<'a, W> {
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

impl<'a, W> Write for Console<'a, W>
where
    W: TextWriter<'a>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.position = self.writer.write_string(self.position, s, self.color);
        Ok(())
    }
}
