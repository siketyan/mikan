use crate::acpi::McfgEntry;

macro_rules! attr_group {
    (#[$a: meta] $($i: item)+) => {
        $(#[$a] $i)+
    }
}

attr_group! {
    #[allow(unused)]

    pub(crate) const COMMAND_INTERRUPT_DISABLE: u16 = 1 << 10;
    pub(crate) const COMMAND_FAST_B2B_ENABLE: u16 = 1 << 9;
    pub(crate) const COMMAND_SERR_ENABLE: u16 = 1 << 8;
    pub(crate) const COMMAND_PARITY_ERROR_RESPONSE: u16 = 1 << 6;
    pub(crate) const COMMAND_VGA_PALETTE_SNOOP: u16 = 1 << 5;
    pub(crate) const COMMAND_MEMORY_WRITE_INVALIDATE_ENABLE: u16 = 1 << 4;
    pub(crate) const COMMAND_SPECIAL_CYCLES: u16 = 1 << 3;
    pub(crate) const COMMAND_BUS_MASTER: u16 = 1 << 2;
    pub(crate) const COMMAND_MEMORY_SPACE: u16 = 1 << 1;
    pub(crate) const COMMAND_IO_SPACE: u16 = 1 << 0;
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct Descriptor {
    // Common Headers
    pub(crate) vendor_id: u16,
    pub(crate) device_id: u16,
    pub(crate) command: u16,
    pub(crate) status: u16,
    pub(crate) revision_id: u8,
    pub(crate) prog_if: u8,
    pub(crate) sub_class: u8,
    pub(crate) class_code: u8,
    pub(crate) cache_line_size: u8,
    pub(crate) latency_timer: u8,
    pub(crate) header_type: u8,
    pub(crate) bist: u8,

    // Base Addresses
    pub(crate) bar0: u32,
    pub(crate) bar1: u32,
    pub(crate) bar2: u32,
    pub(crate) bar3: u32,
    pub(crate) bar4: u32,
    pub(crate) bar5: u32,
}

impl Descriptor {
    pub(crate) fn class(&self) -> u32 {
        ((self.class_code as u32) << 16) + ((self.sub_class as u32) << 8) + (self.prog_if as u32)
    }

    pub(crate) fn bar64_01(&self) -> u64 {
        ((self.bar1 as u64) << 32) | (self.bar0 as u64)
    }

    #[allow(unused)]
    pub(crate) fn bar64_23(&self) -> u64 {
        ((self.bar3 as u64) << 32) | (self.bar2 as u64)
    }

    #[allow(unused)]
    pub(crate) fn bar64_45(&self) -> u64 {
        ((self.bar5 as u64) << 32) | (self.bar4 as u64)
    }
}

macro_rules! impl_common_fns {
    ($t:ty) => {
        impl $t {
            pub(crate) fn descriptor(&self) -> &'static mut Descriptor {
                unsafe { &mut *self.ptr }
            }

            pub(crate) fn is_valid(&self) -> bool {
                self.descriptor().vendor_id != 0xffff
            }
        }
    };
}

pub(crate) struct Function {
    ptr: *mut Descriptor,
}

impl_common_fns!(Function);

#[derive(Copy, Clone)]
pub(crate) struct DeviceIter {
    ptr: *mut Descriptor,
    len: usize,
    cursor: usize,
}

impl Iterator for DeviceIter {
    type Item = Function;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.len {
            let ptr = unsafe { self.ptr.byte_add(self.cursor * 4096) };
            self.cursor += 1;
            Some(Function { ptr })
        } else {
            None
        }
    }
}

pub(crate) struct Device {
    ptr: *mut Descriptor,
}

impl_common_fns!(Device);

impl Device {
    pub(crate) fn iter(&self) -> DeviceIter {
        DeviceIter {
            ptr: self.ptr,
            len: 8,
            cursor: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct BusIter {
    ptr: *mut Descriptor,
    len: usize,
    cursor: usize,
}

impl Iterator for BusIter {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.len {
            let ptr = unsafe { self.ptr.byte_add((self.cursor << 3) * 4096) };
            self.cursor += 1;
            Some(Device { ptr })
        } else {
            None
        }
    }
}

pub(crate) struct Bus {
    ptr: *mut Descriptor,
}

impl_common_fns!(Bus);

impl Bus {
    pub(crate) fn iter(&self) -> BusIter {
        BusIter {
            ptr: self.ptr,
            len: 32,
            cursor: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct ConfigurationIter {
    ptr: *mut Descriptor,
    len: usize,
    cursor: usize,
}

impl Iterator for ConfigurationIter {
    type Item = Bus;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.len {
            let ptr = unsafe { self.ptr.byte_add((self.cursor << 8) * 4096) };
            self.cursor += 1;
            Some(Bus { ptr })
        } else {
            None
        }
    }
}

pub(crate) struct Configuration {
    ptr: *mut Descriptor,
    bus_start: usize,
    bus_end: usize,
}

impl Configuration {
    pub(crate) fn iter(&self) -> ConfigurationIter {
        ConfigurationIter {
            ptr: self.ptr,
            len: self.bus_end - self.bus_start,
            cursor: 0,
        }
    }
}

impl From<McfgEntry> for Configuration {
    fn from(value: McfgEntry) -> Self {
        Self {
            ptr: value.ptr as *mut Descriptor,
            bus_start: value.bus_start as usize,
            bus_end: value.bus_end as usize,
        }
    }
}
