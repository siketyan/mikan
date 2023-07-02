use crate::{Canvas, Colors};

macro_rules! to_byte_arrays {
    ($($l: literal,)*) => {
        &[$(const_str::to_byte_array!($l),)*]
    }
}

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
const MOUSE_CURSOR_SHAPE: &[[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = to_byte_arrays!(
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
);

pub(crate) fn write_cursor<C>(canvas: &mut C)
where
    C: Canvas,
{
    MOUSE_CURSOR_SHAPE.iter().enumerate().for_each(|(y, row)| {
        row.iter().enumerate().for_each(|(x, c)| {
            match c {
                b'@' => canvas
                    .at((200 + x, 100 + y).into())
                    .map(|mut p| p.write(Colors::black())),
                b'.' => canvas
                    .at((200 + x, 100 + y).into())
                    .map(|mut p| p.write(Colors::white())),
                _ => None,
            }
            .unwrap_or(())
        })
    })
}
