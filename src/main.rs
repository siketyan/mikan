#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use core::fmt::Write;
use uefi::prelude::*;

#[entry]
fn main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    if writeln!(system_table.stdout(), "Hello, world!").is_err() {
        return Status::ABORTED;
    }

    #[allow(clippy::empty_loop)]
    loop {}

    #[allow(unreachable_code)]
    Status::SUCCESS
}
