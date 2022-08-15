use core::iter::{Enumerate, Map};
use core::slice::IterMut;
use mikan_core::FrameBufferConfig;

use super::*;

#[inline]
fn pixel_format_to_writer<'a>(pixel_format: PixelFormat) -> &'a dyn PixelWriter {
    match pixel_format {
        PixelFormat::RgbResv8BitPerColor => &RgbPixelWriter,
        PixelFormat::BgrResv8BitPerColor => &BgrPixelWriter,
    }
}

pub(crate) struct FrameBuffer {
    config: FrameBufferConfig,
}

impl Canvas for FrameBuffer {
    #[rustfmt::skip]
    type Pixels<'b> =
        Map<Enumerate<IterMut<'b, [u8; 4]>>, impl FnMut((usize, &'b mut [u8; 4])) -> Pixel>
    where
        Self: 'b;

    fn pixels(&mut self) -> Self::Pixels<'_> {
        let pixels_per_scan_line = self.config.pixels_per_scan_line;
        let pixel_format = self.config.pixel_format;
        unsafe { self.config.buf.as_chunks_unchecked_mut() }
            .iter_mut()
            .enumerate()
            .map(move |(i, buf)| Pixel {
                buf,
                position: Position::from_raw_parts(i, pixels_per_scan_line),
                writer: pixel_format_to_writer(pixel_format),
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
            writer: pixel_format_to_writer(pixel_format),
        })
    }
}

impl From<FrameBufferConfig> for FrameBuffer {
    fn from(config: FrameBufferConfig) -> Self {
        Self { config }
    }
}
