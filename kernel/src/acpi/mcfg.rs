use core::mem::size_of;
use core::ptr::addr_of;

use crate::acpi::{Sdt, SdtHeader};

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct McfgEntry {
    pub(crate) ptr: u64,
    pub(crate) segment: u16,
    pub(crate) bus_start: u8,
    pub(crate) bus_end: u8,
    _reserved: u32,
}

pub(crate) struct McfgIter {
    ptr: *const McfgEntry,
    len: usize,
    cursor: usize,
}

impl Iterator for McfgIter {
    type Item = McfgEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.len {
            let value = unsafe { self.ptr.add(self.cursor) };
            self.cursor += 1;
            Some(unsafe { *value })
        } else {
            None
        }
    }
}

#[repr(C, packed)]
pub(crate) struct Mcfg {
    pub(crate) h: SdtHeader,
    _reserved: u64,
    entries: [McfgEntry; 1],
}

impl Mcfg {
    pub(crate) fn iter(&self) -> McfgIter {
        McfgIter {
            ptr: addr_of!(self.entries) as *const McfgEntry,
            len: (self.h.length as usize - size_of::<SdtHeader>() - size_of::<u64>())
                / size_of::<McfgEntry>(),
            cursor: 0,
        }
    }
}

impl Sdt for Mcfg {}
