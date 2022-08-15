#![no_main]
#![no_std]
#![feature(slice_as_chunks)]

use core::panic::PanicInfo;
use mikan_core::{FrameBufferConfig, KernelArgs, PixelFormat};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

#[derive(Copy, Clone, Debug)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn into_bytes(self, pixel_format: PixelFormat) -> [u8; 4] {
        let Self { r, g, b } = self;
        match pixel_format {
            PixelFormat::RgbResv8BitPerColor => [r, g, b, 0],
            PixelFormat::BgrResv8BitPerColor => [b, g, r, 0],
        }
    }
}

struct Pixel<'a> {
    buf: &'a mut [u8; 4],
    pixel_format: PixelFormat,
}

impl<'a> Pixel<'a> {
    fn write(&mut self, c: Color) {
        *self.buf = c.into_bytes(self.pixel_format);
    }
}

#[derive(Copy, Clone)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn from_raw_parts(index: usize, pixels_per_scan_line: usize) -> Self {
        Self {
            x: index,
            y: pixels_per_scan_line,
        }
    }
}

struct FrameBuffer<'a> {
    config: FrameBufferConfig<'a>,
}

impl<'a> FrameBuffer<'a> {
    fn pixels(&mut self) -> impl Iterator<Item = Pixel> {
        unsafe { self.config.buf.as_chunks_unchecked_mut() }
            .iter_mut()
            .map(|mut buf| Pixel {
                buf,
                pixel_format: self.config.pixel_format,
            })
    }

    fn at(&mut self, Position { x, y }: Position) -> Option<Pixel> {
        let position = self.config.pixels_per_scan_line * y + x;
        self.pixels().nth(position)
    }

    fn fill(&mut self, color: Color) {
        self.pixels().for_each(|mut p| {
            p.write(color);
        });
    }

    fn fill_by<F>(&mut self, f: F)
    where
        F: Fn(Position, usize) -> Color,
    {
        let pixels_per_scan_line = self.config.pixels_per_scan_line;

        self.pixels().enumerate().for_each(|(i, mut p)| {
            p.write(f(Position::from_raw_parts(i, pixels_per_scan_line), i));
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

    frame_buffer.fill(Color::new(0, 0, 0xFF));
    frame_buffer.fill_by(|p, _| {
        let bytes = p.x.to_le_bytes();
        Color::new(bytes[0], bytes[1], bytes[2])
    });

    // frame_buffer
    //     .config
    //     .buf
    //     .iter_mut()
    //     .enumerate()
    //     .for_each(|(i, p)| {
    //         *p = (i % 256) as u8;
    //     });

    loop {
        aarch64::instructions::halt();
    }
}
