#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]
#![feature(type_alias_impl_trait)]

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

static mut FRAME_BUFFER: Option<FrameBuffer> = None;
static mut CONSOLE: Option<Console<FrameBuffer>> = None;

macro_rules! println {
    ($($t: tt)*) => {
        if let Some(c) = unsafe { CONSOLE.as_mut() } {
            writeln!(c, $($t)*).ok();
        }
    };
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_main(args: KernelArgs) -> ! {
    unsafe {
        FRAME_BUFFER = Some(FrameBuffer::from(args.frame_buffer));
        CONSOLE = Some(
            Console::new(FRAME_BUFFER.as_mut().unwrap())
                .with_position((0, 82))
                .with_color(Colors::black()),
        );
    }

    let frame_buffer = unsafe { FRAME_BUFFER.as_mut().unwrap() };

    frame_buffer.fill(Colors::white());
    frame_buffer.fill_in(Region::new((100, 100).into(), 200, 100), Colors::green());
    frame_buffer.write_chars((0, 50).into(), '!'..='~', Colors::black());
    frame_buffer.write_string((0, 66).into(), "Hello, world!", Colors::blue());

    println!("1 + 2 = {}", 1 + 2);
    println!(
        "It's so Loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong string"
    );

    (0..30).for_each(|i| println!("line {}", i));

    loop {
        aarch64::instructions::halt();
    }
}
