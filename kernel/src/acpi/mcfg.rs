use core::ffi::{c_char, c_void};

use crate::acpi::Sdt;

#[repr(C, packed)]
pub(crate) struct Mcfg {
    signature: [c_char; 4],
}

impl Sdt for Mcfg {}
