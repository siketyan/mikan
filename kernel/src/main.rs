#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

#[no_mangle]
extern "C" fn kernel_main() {
    loop {
        aarch64::instructions::halt();
    }
}
