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
use crate::graphics::cursor::write_cursor;
use crate::graphics::frame_buffer::FrameBuffer;
use crate::graphics::text::TextWriter;
use crate::graphics::{Canvas, Color, Colors, Position, Region};

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
    let background = Color::new(45, 118, 237);
    let foreground = Colors::white();

    unsafe {
        FRAME_BUFFER = Some(FrameBuffer::from(args.frame_buffer));
        CONSOLE = Some(
            Console::new(FRAME_BUFFER.as_mut().unwrap())
                .with_position((0, 0))
                .with_background(background)
                .with_foreground(foreground),
        );
    }

    let frame_buffer = unsafe { FRAME_BUFFER.as_mut().unwrap() };
    let (width, height) = (frame_buffer.width(), frame_buffer.height());

    frame_buffer.fill_in(
        Region::new(Position::zero(), width, height - 50),
        background,
    );

    frame_buffer.fill_in(
        Region::new((0, height - 50), width, 50),
        Color::new(1, 8, 17),
    );

    frame_buffer.fill_in(
        Region::new((0, height - 50), width / 5, 50),
        Color::new(80, 80, 80),
    );

    frame_buffer.fill_in(
        Region::new((10, height - 40), 30, 30),
        Color::new(160, 160, 160),
    );

    println!("Welcome to MikanOS!");

    write_cursor(frame_buffer);

    loop {
        aarch64::instructions::halt();
    }
}
