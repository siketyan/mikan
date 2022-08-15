#![no_main]
#![no_std]

use core::panic::PanicInfo;
use mikan_core::KernelArgs;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_main(args: KernelArgs) -> ! {
    let frame_buffer = args.frame_buffer;

    frame_buffer
        .buf
        .iter_mut()
        .enumerate()
        .for_each(|(i, buf)| {
            *buf = (i % 256) as u8;
        });

    loop {
        aarch64::instructions::halt();
    }
}
