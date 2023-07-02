mod mcfg;
mod rsdp;
mod xsdt;

use core::ffi::c_char;

pub(crate) use mcfg::*;
pub(crate) use rsdp::*;
pub(crate) use xsdt::*;

pub(crate) trait Sdt {}

#[repr(C, packed)]
pub(crate) struct SdtHeader {
    pub(crate) signature: [c_char; 4],
    pub(crate) length: u32,
    pub(crate) revision: u8,
    pub(crate) checksum: u8,
    pub(crate) oem_id: [c_char; 6],
    pub(crate) oem_table_id: [c_char; 8],
    pub(crate) oem_revision: u32,
    pub(crate) creator_id: u32,
    pub(crate) creator_revision: u32,
}
