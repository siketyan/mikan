#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

mod graphics;

#[cfg(not(test))]
use core::panic::PanicInfo;
use mikan_core::KernelArgs;

use crate::graphics::colors::Colors;
use crate::graphics::frame_buffer::FrameBuffer;
use crate::graphics::text::TextWriter;
use crate::graphics::{Canvas, Region};

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_main(args: KernelArgs) -> ! {
    let mut frame_buffer = FrameBuffer::from(args.frame_buffer);

    frame_buffer.fill(Colors::white());
    frame_buffer.fill_in(Region::new((100, 100).into(), 200, 100), Colors::green());
    frame_buffer.write_chars((0, 50).into(), '!'..='~', Colors::black());
    frame_buffer.write_string((0, 66).into(), "Hello, world!", Colors::blue());

    loop {
        aarch64::instructions::halt();
    }
}
