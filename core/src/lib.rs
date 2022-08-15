#![no_std]

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    RgbResv8BitPerColor,
    BgrResv8BitPerColor,
}

#[derive(Debug)]
pub struct FrameBufferConfig {
    pub buf: &'static mut [u8],
    pub pixels_per_scan_line: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_format: PixelFormat,
}

#[derive(Debug)]
pub struct KernelArgs {
    pub frame_buffer: FrameBufferConfig,
}

pub type Entrypoint = extern "C" fn(KernelArgs) -> !;
