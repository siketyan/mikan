use mikan_core::PixelFormat;

pub(crate) mod frame_buffer;

const PIXEL_SIZE: usize = 4;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn into_bytes(self, pixel_format: PixelFormat) -> [u8; PIXEL_SIZE] {
        let Self { r, g, b } = self;
        match pixel_format {
            PixelFormat::RgbResv8BitPerColor => [r, g, b, 0],
            PixelFormat::BgrResv8BitPerColor => [b, g, r, 0],
        }
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self::new(
            (value >> 16 & 0xFF) as u8,
            (value >> 8 & 0xFF) as u8,
            (value & 0xFF) as u8,
        )
    }
}

pub(crate) struct Pixel<'a> {
    buf: &'a mut [u8; PIXEL_SIZE],
    position: Position,
    pixel_format: PixelFormat,
}

impl<'a> Pixel<'a> {
    fn write(&mut self, c: Color) {
        *self.buf = c.into_bytes(self.pixel_format);
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn from_raw_parts(index: usize, pixels_per_scan_line: usize) -> Self {
        Self {
            x: index % pixels_per_scan_line,
            y: index / pixels_per_scan_line,
        }
    }

    fn into_raw_parts(self, pixels_per_scan_line: usize) -> usize {
        self.y * pixels_per_scan_line + self.x
    }

    fn into_offset(self, pixels_per_scan_line: usize) -> usize {
        self.into_raw_parts(pixels_per_scan_line) * PIXEL_SIZE
    }
}

impl From<(usize, usize)> for Position {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Region {
    position: Position,
    width: usize,
    height: usize,
}

impl Region {
    pub(crate) fn new(position: Position, width: usize, height: usize) -> Self {
        Self {
            position,
            width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_rgb() {
        assert_eq!(Color::new(0x12, 0x34, 0x56), Color::from(0x123456));
    }
}
