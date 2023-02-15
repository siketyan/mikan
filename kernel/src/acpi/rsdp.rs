use core::ffi::{c_char, c_void};
use crate::acpi::Xsdt;

#[repr(C, packed)]
pub(crate) struct RsdpDescriptor {
    pub(crate) signature: [c_char; 8],
    pub(crate) checksum: u8,
    pub(crate) oem_id: [c_char; 6],
    pub(crate) revision: u8,
    pub(crate) rsdt_address: u32,
}

#[repr(C, packed)]
pub(crate) struct RsdpDescriptor20 {
    pub(crate) first_part: RsdpDescriptor,
    pub(crate) length: u32,
    pub(crate) xsdt_address: u64,
    pub(crate) extended_checksum: u8,
    pub(crate) reserved: [u8; 3],
}

impl RsdpDescriptor {
    pub(crate) fn from_ptr(ptr: *const c_void) -> &'static Self {
        unsafe { &*(ptr as *const Self) }
    }

    pub(crate) fn as_rev_2(&'static self) -> Option<&'static RsdpDescriptor20> {
        if self.revision >= 2 {
            Some(RsdpDescriptor20::from_ptr(
                self as *const Self as *const c_void,
            ))
        } else {
            None
        }
    }
}

impl RsdpDescriptor20 {
    fn from_ptr(ptr: *const c_void) -> &'static Self {
        unsafe { &*(ptr as *const Self) }
    }

    pub fn xsdt(&self) -> &'static Xsdt {
        self.xsdt_address.into()
    }
}
