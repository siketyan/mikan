#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

mod graphics;

#[cfg(not(test))]
use core::panic::PanicInfo;
use mikan_core::KernelArgs;

use crate::graphics::frame_buffer::FrameBuffer;
use crate::graphics::text::TextWriter;
use crate::graphics::{Canvas, Color, Region};

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
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

    frame_buffer.write_ascii((50, 50).into(), 'A', 0x000000.into());
    frame_buffer.write_ascii((58, 50).into(), 'B', 0x000000.into());
    frame_buffer.write_ascii((66, 50).into(), 'C', 0x000000.into());

    loop {
        aarch64::instructions::halt();
    }
}
