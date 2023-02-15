use core::arch::asm;

const MMIO_BASE_ADDRESS: usize = 0x1000_0000;
const SMC_PCI_VERSION: u32 = 0x8400_0130;
const SMC_SMCCC_VERSION: u32 = 0x8000_0000;

#[derive(Debug)]
#[repr(C)]
pub(crate) struct CommonHeader {
    device_id: u16,
    vendor_id: u16,
    status: u16,
    command: u16,
    class_code: u8,
    sub_class: u8,
    prog_if: u8,
    revision_id: u8,
    bist: u8,
    header_type: u8,
    latency_timer: u8,
    cache_line_size: u8,
}

type FnPciVersion = extern "C" fn() -> u32;

pub(crate) fn make_address(bus: u8, device: u8, function: u8, address: u8) -> usize {
    MMIO_BASE_ADDRESS
        + ((bus as usize) << 20
            | (device as usize) << 15
            | (function as usize) << 12
            | (address as usize))
}

pub(crate) fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    unsafe { &*(make_address(bus, device, function, 0x00) as *mut CommonHeader) }.vendor_id
}

pub(crate) fn read_header<'a>() -> &'a CommonHeader {
    unsafe { &*(make_address(0, 0, 0, 0x00) as *mut CommonHeader) }
}

pub(crate) fn pci_version() -> (u16, u16) {
    let mut version = 0u32;
    // unsafe {
    //     asm!("mov w0, {id:w}", id = in(reg) SMC_SMCCC_VERSION);
    //     asm!("smc 0", clobber_abi("C"));
    //     asm!("mov {version:w}, w0", version = out(reg) version);
    // }

    ((version >> 16) as u16, (version & 0xffff) as u16)
}
