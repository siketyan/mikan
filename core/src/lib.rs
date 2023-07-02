#![no_std]

use core::ffi::c_void;

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
pub struct AcpiConfig {
    pub rsdp_address: *const c_void,
}

#[derive(Debug)]
pub struct KernelArgs {
    pub frame_buffer: FrameBufferConfig,
    pub acpi: AcpiConfig,
}

pub type Entrypoint = extern "C" fn(KernelArgs) -> !;
