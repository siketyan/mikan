#![no_main]
#![no_std]
#![feature(abi_efiapi)]

mod buf;

extern crate alloc;

use alloc::format;
use anyhow::{anyhow, Result};
use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::console::text::Output;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, RegularFile};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi::CString16;

use crate::buf::allocate_aligned;

macro_rules! err {
    () => {
        |e| anyhow!(e)
    };
    ($msg: literal $(, $v: expr)*) => {
        |e| anyhow!("{}: {:?}", format!($msg $(, $v)*), e)
    };
}

const KERNEL_ADDRESS: usize = 0x40000000;
const KERNEL_ENTRYPOINT_ADDRESS: usize = KERNEL_ADDRESS + 24;

type Entrypoint = extern "C" fn();

struct WrappedFile {
    file: RegularFile,
}

impl WrappedFile {
    fn close(self) {
        self.file.close()
    }
}

impl From<RegularFile> for WrappedFile {
    fn from(file: RegularFile) -> Self {
        Self { file }
    }
}

impl Write for WrappedFile {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.file.write(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

struct Application {
    handle: Handle,
    system_table: SystemTable<Boot>,
}

impl Application {
    fn new(handle: Handle, system_table: SystemTable<Boot>) -> Self {
        Self {
            handle,
            system_table,
        }
    }

    fn stdout(&mut self) -> &mut Output {
        self.system_table.stdout()
    }

    fn load_file(&mut self, root_dir: &mut Directory, path: &str) -> Result<RegularFile> {
        root_dir
            .open(
                CString16::try_from(path)
                    .map_err(err!("Invalid path"))?
                    .as_ref(),
                FileMode::CreateReadWrite,
                FileAttribute::empty(),
            )
            .map_err(|_| anyhow!("Failed to create a file"))?
            .into_regular_file()
            .ok_or_else(|| anyhow!("The file was not a regular file"))
    }

    fn save_memory_map<'a, I>(&mut self, memory_map: I, root_dir: &mut Directory) -> Result<()>
    where
        I: Iterator<Item = &'a MemoryDescriptor>,
    {
        let mut file = self
            .load_file(root_dir, "\\memmap")
            .map(WrappedFile::from)?;

        writeln!(file, "Index, Type, PhysicalStart, NumberOfPages, Attribute").map_err(err!())?;

        memory_map
            .enumerate()
            .try_for_each(|(i, descriptor)| {
                writeln!(
                    file,
                    "{:?}, {:?}, {:#x}, {:?}, {:?}",
                    i, descriptor.ty, descriptor.phys_start, descriptor.page_count, descriptor.att
                )
            })
            .map_err(err!())?;

        file.close();

        writeln!(self.stdout(), "Saved the memory map to \\memmap").map_err(err!())
    }

    fn load_kernel(&mut self, root_dir: &mut Directory) -> Result<()> {
        let mut file = self.load_file(root_dir, "\\kernel.elf")?;
        let mut buffer = allocate_aligned::<FileInfo>(14);
        let info: &mut FileInfo = file
            .get_info(&mut buffer)
            .map_err(|e| anyhow!("Failed to get information of the file: {:?}", e))?;
        let kernel_size = info.file_size();

        writeln!(self.stdout(), "Kernel size is {} bytes", kernel_size).map_err(err!())?;

        self.system_table
            .boot_services()
            .allocate_pages(
                AllocateType::Address(KERNEL_ADDRESS),
                MemoryType::LOADER_DATA,
                ((kernel_size + 0xfff) / 0x1000) as usize,
            )
            .map_err(err!(
                "Failed to allocate {} bytes at {:#x}",
                kernel_size,
                KERNEL_ADDRESS
            ))?;

        file.read(unsafe {
            core::slice::from_raw_parts_mut(KERNEL_ADDRESS as *mut u8, kernel_size as usize)
        })
        .map_err(err!("Failed to read kernel from the file"))?;

        writeln!(
            self.stdout(),
            "Loaded kernel to {:#x} ({} bytes)",
            KERNEL_ADDRESS,
            kernel_size
        )
        .map_err(err!())
    }

    fn boot(mut self) -> Result<()> {
        writeln!(self.stdout(), "Booting kernel, exiting boot services").map_err(err!())?;

        self.system_table
            .exit_boot_services(
                self.handle,
                allocate_aligned::<MemoryDescriptor>(4096).as_mut(),
            )
            .map(|_| ())
            .map_err(|_| anyhow!("Could not exit boot services"))?;

        (unsafe { (KERNEL_ENTRYPOINT_ADDRESS as *mut Entrypoint).read() })();
        Ok(())
    }

    fn execute(mut self) -> Result<()> {
        writeln!(self.system_table.stdout(), "Hello, world!").map_err(err!())?;

        let boot_services = self.system_table.boot_services();

        let mut buffer = allocate_aligned::<MemoryDescriptor>(4096);
        let (_key, iter) = boot_services
            .memory_map(&mut buffer)
            .map_err(err!("Could not get the memory map"))?;

        let mut root_dir: Directory = boot_services
            .get_image_file_system(self.handle)
            .map_err(err!("Could not get a filesystem from the image"))
            .and_then(|protocol| {
                unsafe { protocol.interface.get().as_mut() }
                    .ok_or_else(|| anyhow!("Could not get filesystem protocol"))
            })?
            .open_volume()
            .map_err(err!("Failed to open a volume"))?;

        self.save_memory_map(iter, &mut root_dir)?;
        self.load_kernel(&mut root_dir)?;

        self.boot()
    }
}

fn try_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Result<()> {
    uefi_services::init(&mut system_table).map_err(err!("Failed to initialise UEFI services"))?;

    Application::new(handle, system_table).execute()?;

    #[allow(clippy::empty_loop)]
    loop {}
}

#[entry]
fn main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    match try_main(handle, system_table) {
        Ok(_) => Status::SUCCESS,
        Err(e) => {
            panic!("{}", e);
        }
    }
}
