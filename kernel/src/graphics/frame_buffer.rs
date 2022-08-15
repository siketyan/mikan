use mikan_core::FrameBufferConfig;

use super::*;

#[inline]
fn pixel_format_to_writer<'a>(pixel_format: PixelFormat) -> &'a dyn PixelWriter {
    match pixel_format {
        PixelFormat::RgbResv8BitPerColor => &RgbPixelWriter,
        PixelFormat::BgrResv8BitPerColor => &BgrPixelWriter,
    }
}

pub(crate) struct FrameBuffer<'a> {
    config: FrameBufferConfig<'a>,
}

impl<'a> FrameBuffer<'a> {
    pub(crate) fn pixels(&mut self) -> impl Iterator<Item = Pixel> {
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

    pub(crate) fn at(&mut self, position: Position) -> Option<Pixel> {
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

    pub(crate) fn fill(&mut self, color: Color) {
        self.pixels().for_each(|mut p| p.write(color));
    }

    pub(crate) fn fill_in(&mut self, region: Region, color: Color) {
        let Position { x, y } = region.position;
        for y in y..(y + region.height) {
            for x in x..(x + region.width) {
                if let Some(mut p) = self.at(Position { x, y }) {
                    p.write(color);
                }
            }
        }
    }

    pub(crate) fn fill_by<F>(&mut self, f: F)
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
