use crate::acpi::McfgEntry;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CommonHeader {
    pub(crate) vendor_id: u16,
    pub(crate) device_id: u16,
    pub(crate) status: u16,
    pub(crate) command: u16,
    pub(crate) revision_id: u8,
    pub(crate) prog_if: u8,
    pub(crate) sub_class: u8,
    pub(crate) class_code: u8,
    pub(crate) cache_line_size: u8,
    pub(crate) latency_timer: u8,
    pub(crate) header_type: u8,
    pub(crate) bist: u8,
}

impl CommonHeader {
    pub(crate) fn class(&self) -> u32 {
        ((self.class_code as u32) << 16) + ((self.sub_class as u32) << 8) + (self.prog_if as u32)
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub(crate) struct Descriptor {
    pub(crate) h: CommonHeader,
}

macro_rules! impl_common_fns {
    ($t:ty) => {
        impl $t {
            pub(crate) fn descriptor(&self) -> &'static Descriptor {
                unsafe { &*self.ptr }
            }

            pub(crate) fn is_valid(&self) -> bool {
                self.descriptor().h.vendor_id != 0xffff
            }
        }
    };
}

pub(crate) struct Function {
    ptr: *const Descriptor,
}

impl_common_fns!(Function);

#[derive(Copy, Clone)]
pub(crate) struct DeviceIter {
    ptr: *const Descriptor,
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
    ptr: *const Descriptor,
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
    ptr: *const Descriptor,
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
    ptr: *const Descriptor,
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
    ptr: *const Descriptor,
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
    ptr: *const Descriptor,
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
            ptr: value.ptr as *const Descriptor,
            bus_start: value.bus_start as usize,
            bus_end: value.bus_end as usize,
        }
    }
}
