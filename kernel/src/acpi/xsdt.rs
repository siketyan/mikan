use core::ffi::{c_char, c_void};
use core::mem::size_of;
use core::ptr::addr_of;

use crate::acpi::Sdt;
use crate::println;

#[repr(C, packed)]
pub(crate) struct AcpiSdtHeader {
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

pub(crate) struct XsdtEntry {
    ptr: *const c_void,
}

impl XsdtEntry {
    pub(crate) fn signature(&self) -> &'static [c_char; 4] {
        unsafe { &*(self.ptr as *const [c_char; 4]) }
    }

    pub(crate) fn as_sdt<T>(&self) -> &'static T
    where
        T: Sdt,
    {
        unsafe { &*(self.ptr as *const T) }
    }
}

pub(crate) struct XsdtIter {
    ptr: *const u64,
    len: usize,
    cursor: usize,
}

impl Iterator for XsdtIter {
    type Item = XsdtEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.len {
            let value = unsafe { self.ptr.add(self.cursor) };
            self.cursor += 1;
            Some(XsdtEntry {
                ptr: unsafe { *value } as *const c_void,
            })
        } else {
            None
        }
    }
}

#[repr(C, packed)]
pub(crate) struct Xsdt {
    pub(crate) h: AcpiSdtHeader,
    pointer_to_other_sdt: [u64; 1],
}

impl Xsdt {
    pub(crate) fn iter(&self) -> XsdtIter {
        XsdtIter {
            ptr: addr_of!(self.pointer_to_other_sdt) as *const u64,
            len: (self.h.length as usize - size_of::<AcpiSdtHeader>()) / size_of::<u64>(),
            cursor: 0,
        }
    }

    pub(crate) fn find_sdt<T>(&self, signature: &[c_char; 4]) -> Option<&'static T>
    where
        T: Sdt,
    {
        self.iter()
            .find(|e| e.signature() == signature)
            .map(|e| e.as_sdt::<T>())
    }
}

impl From<u64> for &'static Xsdt {
    fn from(value: u64) -> Self {
        unsafe { &*(value as *const c_void as *const Xsdt) }
    }
}
