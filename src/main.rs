#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate alloc;

use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use core::fmt::Write;
use core::mem::align_of;
use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileMode, RegularFile};
use uefi::table::boot::MemoryDescriptor;
use uefi::CString16;

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

    fn save_memory_map<'a, I>(&self, memory_map: I, file: &mut WrappedFile) -> core::fmt::Result
    where
        I: Iterator<Item = &'a MemoryDescriptor>,
    {
        writeln!(file, "Index, Type, PhysicalStart, NumberOfPages, Attribute")?;

        memory_map.enumerate().try_for_each(|(i, descriptor)| {
            writeln!(
                file,
                "{:?}, {:?}, {:?}, {:?}, {:?}",
                i, descriptor.ty, descriptor.phys_start, descriptor.page_count, descriptor.att
            )
        })
    }

    fn execute(&mut self) -> Result<()> {
        writeln!(self.system_table.stdout(), "Hello, world!").map_err(|e| anyhow!(e))?;

        let boot_services = self.system_table.boot_services();
        let mut memory_map = Vec::<u8>::with_capacity(4096 * align_of::<MemoryDescriptor>());
        #[allow(clippy::uninit_vec)]
        unsafe {
            memory_map.set_len(memory_map.capacity());
        }

        let (_key, iter) = boot_services
            .memory_map(&mut memory_map)
            .map_err(|e| anyhow!("Could not get the memory map: {:?}", e))?;

        let mut root_dir: Directory = boot_services
            .get_image_file_system(self.handle)
            .map_err(|_| anyhow!("Could not get a filesystem from the image"))
            .and_then(|protocol| {
                unsafe { protocol.interface.get().as_mut() }
                    .ok_or_else(|| anyhow!("Could not get filesystem protocol"))
            })?
            .open_volume()
            .map_err(|_| anyhow!("Failed to open a volume"))?;

        let mut file = root_dir
            .open(
                CString16::try_from("\\memmap")
                    .map_err(|_| anyhow!("Invalid path"))?
                    .as_ref(),
                FileMode::CreateReadWrite,
                FileAttribute::empty(),
            )
            .map_err(|_| anyhow!("Failed to create a file"))?
            .into_regular_file()
            .map(WrappedFile::from)
            .ok_or_else(|| anyhow!("The file was not a regular file"))?;

        self.save_memory_map(iter, &mut file)
            .map_err(|_| anyhow!("Failed to save memory map into the file"))?;

        file.close();

        writeln!(
            self.system_table.stdout(),
            "Saved the memory file to \\memmap"
        )
        .map_err(|e| anyhow!(e))?;

        Ok(())
    }
}

fn try_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Result<()> {
    uefi_services::init(&mut system_table)
        .map_err(|_| anyhow!("Failed to initialise UEFI services"))?;

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
