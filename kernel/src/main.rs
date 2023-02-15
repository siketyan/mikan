#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(slice_as_chunks)]
#![feature(type_alias_impl_trait)]
#![feature(pointer_byte_offsets)]

mod acpi;
mod console;
mod graphics;
mod pci;

use core::ffi::c_char;
#[cfg(not(test))]
use core::panic::PanicInfo;

use mikan_core::KernelArgs;

use crate::acpi::{Mcfg, RsdpDescriptor};
use crate::console::Console;
use crate::graphics::cursor::write_cursor;
use crate::graphics::frame_buffer::FrameBuffer;
use crate::graphics::text::TextWriter;
use crate::graphics::{Canvas, Color, Colors, Position, Region};
use crate::pci::{CommonHeader, Configuration};

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

static mut FRAME_BUFFER: Option<FrameBuffer> = None;
static mut CONSOLE: Option<Console<FrameBuffer>> = None;

#[macro_export]
macro_rules! println {
    ($($t: tt)*) => {
        if let Some(c) = unsafe { $crate::CONSOLE.as_mut() } {
            use core::fmt::Write;
            writeln!(c, $($t)*).ok();
        }
    };
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn kernel_main(args: KernelArgs) -> ! {
    let background = Color::new(45, 118, 237);
    let foreground = Colors::white();

    unsafe {
        FRAME_BUFFER = Some(FrameBuffer::from(args.frame_buffer));
        CONSOLE = Some(
            Console::new(FRAME_BUFFER.as_mut().unwrap())
                .with_position((0, 0))
                .with_background(background)
                .with_foreground(foreground),
        );
    }

    let frame_buffer = unsafe { FRAME_BUFFER.as_mut().unwrap() };
    let (width, height) = (frame_buffer.width(), frame_buffer.height());

    frame_buffer.fill_in(
        Region::new(Position::zero(), width, height - 50),
        background,
    );

    frame_buffer.fill_in(
        Region::new((0, height - 50), width, 50),
        Color::new(1, 8, 17),
    );

    frame_buffer.fill_in(
        Region::new((0, height - 50), width / 5, 50),
        Color::new(80, 80, 80),
    );

    frame_buffer.fill_in(
        Region::new((10, height - 40), 30, 30),
        Color::new(160, 160, 160),
    );

    println!("Welcome to MikanOS!");
    println!("RSDP is at {:?}", args.acpi.rsdp_address);

    let rsdp = RsdpDescriptor::from_ptr(args.acpi.rsdp_address);
    let rsdp2 = rsdp.as_rev_2().unwrap();

    println!("RSDP Revision: {}", rsdp.revision);
    println!("RSDP2 XSDT Address: {:X}", rsdp.rsdt_address as usize);

    let xsdt = rsdp2.xsdt();

    println!("XSDT Length: {}", xsdt.h.length as usize);
    println!("XSDT Revision: {}", xsdt.h.revision);

    let mcfg = xsdt.find_sdt::<Mcfg>(&[
        b'M' as c_char,
        b'C' as c_char,
        b'F' as c_char,
        b'G' as c_char,
    ]);

    println!("MCFG is at: {:?}", mcfg.map(|m| m as *const Mcfg));

    let e = mcfg.unwrap().iter().next().unwrap();
    println!("PCI Configuration Entry:");
    println!("  Base Address: {:X}", e.ptr as usize);
    println!("  Segment Group: {}", e.segment as usize);
    println!("  Bus Start: {}", e.bus_start);
    println!("  Bus End: {}", e.bus_end);

    let cfg = Configuration::from(e);
    let iter = cfg
        .iter()
        .filter(|b| b.is_valid())
        .enumerate()
        .flat_map(|(i, bus)| {
            bus.iter()
                .filter(|d| d.is_valid())
                .enumerate()
                .map(move |(j, device)| (i, j, device))
        })
        .flat_map(|(i, j, device)| {
            device
                .iter()
                .filter(|f| f.is_valid())
                .enumerate()
                .map(move |(k, function)| (i, j, k, function))
        });

    iter.clone().for_each(|(i, j, k, function)| {
        let h = function.descriptor().h;
        println!(
            "{}.{}.{}: vend {:04X}, dev {:04X}, class {:06X}, head {:02X}",
            i,
            j,
            k,
            h.vendor_id as usize,
            h.device_id as usize,
            h.class(),
            h.header_type as usize,
        );
    });

    let xhc = iter
        .clone()
        .map(|(_, _, _, f)| f)
        .find(|f| f.descriptor().h.class() == 0x0c0330);

    if xhc.is_some() {
        println!("xHC Controller Found: {:?}", xhc.unwrap().descriptor());
    }

    write_cursor(frame_buffer);

    loop {
        aarch64::instructions::halt();
    }
}
