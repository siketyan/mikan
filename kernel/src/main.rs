#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

mod console;
mod graphics;

#[cfg(not(test))]
use core::panic::PanicInfo;

use core::fmt::Write;
use mikan_core::KernelArgs;

use crate::console::Console;
use crate::graphics::frame_buffer::FrameBuffer;
use crate::graphics::text::TextWriter;
use crate::graphics::{Canvas, Colors, Region};

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

    let mut c = Console::new(&mut frame_buffer)
        .with_position((0, 82))
        .with_color(Colors::black());

    writeln!(c, "1 + 2 = {}", 1 + 2).ok();
    writeln!(
        c,
        "It's so Loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong string"
    )
    .ok();

    (0..30).try_for_each(|i| writeln!(c, "line {}", i)).ok();

    loop {
        aarch64::instructions::halt();
    }
}
