#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

#[no_mangle]
extern "C" fn kernel_main(frame_buffer_ptr: *mut u8, frame_buffer_size: usize) -> ! {
    let frame_buffer =
        unsafe { core::slice::from_raw_parts_mut(frame_buffer_ptr, frame_buffer_size) };

    frame_buffer.iter_mut().enumerate().for_each(|(i, buf)| {
        *buf = (i % 256) as u8;
    });

    loop {
        aarch64::instructions::halt();
    }
}
