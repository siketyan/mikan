#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]

#[cfg(not(test))]
use core::panic::PanicInfo;
use mikan_core::{FrameBufferConfig, KernelArgs, PixelFormat};

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

const PIXEL_SIZE: usize = 4;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Color {
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

#[cfg(test)]
mod tests {
    use crate::Color;

    #[test]
    fn color_from_rgb() {
        assert_eq!(Color::new(0x12, 0x34, 0x56), Color::from(0x123456));
    }
}

struct Pixel<'a> {
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
struct Position {
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
struct Region {
    position: Position,
    width: usize,
    height: usize,
}

impl Region {
    fn new(position: Position, width: usize, height: usize) -> Self {
        Self {
            position,
            width,
            height,
        }
    }
}

struct FrameBuffer<'a> {
    config: FrameBufferConfig<'a>,
}

impl<'a> FrameBuffer<'a> {
    fn pixels(&mut self) -> impl Iterator<Item = Pixel> {
        let pixels_per_scan_line = self.config.pixels_per_scan_line;
        let pixel_format = self.config.pixel_format;
        unsafe { self.config.buf.as_chunks_unchecked_mut() }
            .iter_mut()
            .enumerate()
            .map(move |(i, buf)| Pixel {
                buf,
                position: Position::from_raw_parts(i, pixels_per_scan_line),
                pixel_format,
            })
    }

    fn at(&mut self, position: Position) -> Option<Pixel> {
        let offset = position.into_offset(self.config.pixels_per_scan_line);
        let pixel_format = self.config.pixel_format;

        Some(Pixel {
            buf: (&mut self.config.buf[offset..offset + PIXEL_SIZE])
                .try_into()
                .ok()?,
            position,
            pixel_format,
        })
    }

    fn fill(&mut self, color: Color) {
        self.pixels().for_each(|mut p| p.write(color));
    }

    fn fill_in(&mut self, region: Region, color: Color) {
        let Position { x, y } = region.position;
        for y in y..(y + region.height) {
            for x in x..(x + region.width) {
                if let Some(mut p) = self.at(Position { x, y }) {
                    p.write(color);
                }
            }
        }
    }

    fn fill_by<F>(&mut self, f: F)
    where
        F: Fn(Position) -> Color,
    {
        self.pixels().for_each(|mut p| {
            p.write(f(p.position));
        });
    }
}

impl<'a> From<FrameBufferConfig<'a>> for FrameBuffer<'a> {
    fn from(config: FrameBufferConfig<'a>) -> Self {
        Self { config }
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_main(args: KernelArgs) -> ! {
    let mut frame_buffer = FrameBuffer::from(args.frame_buffer);

    frame_buffer.fill(Color::from(0xFFFFFF));
    frame_buffer.fill_in(
        Region::new((100, 100).into(), 200, 100),
        Color::from(0x00FF00),
    );

    loop {
        aarch64::instructions::halt();
    }
}
