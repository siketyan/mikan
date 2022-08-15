#![no_main]
#![no_std]
#![feature(abi_efiapi)]

mod buf;

extern crate alloc;

use alloc::format;
use anyhow::{anyhow, Result};
use core::cmp::{max, min};
use core::fmt::Write;
use elf_rs::{Elf, ElfFile, ProgramType};
use mikan_core::{Entrypoint, FrameBufferConfig, KernelArgs};
use uefi::prelude::*;
use uefi::proto::console::gop::{FrameBuffer, GraphicsOutput, ModeInfo, PixelFormat};
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, RegularFile};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType, SearchType};
use uefi::CString16;
use uefi::Identify;

use crate::buf::{allocate_aligned, allocate_uninit};

macro_rules! err {
    () => {
        |e| anyhow!(e)
    };
    ($msg: literal $(, $v: expr)*) => {
        |e| anyhow!("{}: {:?}", format!($msg $(, $v)*), e)
    };
}

macro_rules! println {
    ($($t: tt)*) => {
        writeln!(unsafe { uefi_services::system_table().as_mut() }.stdout(), $($t)*)
            .map_err(err!())
    };
}

macro_rules! eprintln {
    ($($t: tt)*) => {
        writeln!(unsafe { uefi_services::system_table().as_mut() }.stderr(), $($t)*)
            .unwrap_or(())
    };
}

const KERNEL_ADDRESS: usize = 0x40000000;

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

    fn load_file(&mut self, root_dir: &mut Directory, path: &str) -> Result<RegularFile> {
        root_dir
            .open(
                CString16::try_from(path)
                    .map_err(err!("Invalid path"))?
                    .as_ref(),
                FileMode::CreateReadWrite,
                FileAttribute::empty(),
            )
            .map_err(err!("Failed to create a file"))?
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

        println!("Saved the memory map to \\memmap")?;
        Ok(())
    }

    fn get_frame_buffer(&mut self) -> Result<FrameBufferConfig> {
        let mut handles = allocate_uninit(16);

        self.system_table
            .boot_services()
            .locate_handle(
                SearchType::ByProtocol(&GraphicsOutput::GUID),
                Some(&mut handles),
            )
            .map_err(err!("Failed to locate GOP handle"))?;

        let handle = handles
            .into_iter()
            .map(|h| unsafe { h.assume_init() })
            .next()
            .ok_or_else(|| anyhow!("No GOP handles available"))?;

        #[allow(deprecated)]
        let gop: &mut GraphicsOutput = self
            .system_table
            .boot_services()
            .handle_protocol::<GraphicsOutput>(handle)
            .map_err(err!("Failed to open graphics output protocol"))
            .and_then(|protocol| {
                unsafe { protocol.get().as_mut() }
                    .ok_or_else(|| anyhow!("Could not get the protocol"))
            })?;

        let mode_info: ModeInfo = gop.current_mode_info();
        let (width, height) = mode_info.resolution();

        println!(
            "Resolution: {}x{}, Pixel Format: {:?}, {} pixels/line",
            width,
            height,
            mode_info.pixel_format(),
            mode_info.stride()
        )?;

        let mut frame_buffer: FrameBuffer = gop.frame_buffer();
        let frame_buffer_ptr = frame_buffer.as_mut_ptr() as usize;

        println!(
            "Frame Buffer: {:#x} - {:#x}, Size: {} bytes",
            frame_buffer_ptr,
            frame_buffer_ptr + frame_buffer.size(),
            frame_buffer.size()
        )?;

        Ok(FrameBufferConfig {
            buf: unsafe {
                core::slice::from_raw_parts_mut(frame_buffer.as_mut_ptr(), frame_buffer.size())
            },
            pixels_per_scan_line: mode_info.stride(),
            width,
            height,
            pixel_format: match mode_info.pixel_format() {
                PixelFormat::Rgb => mikan_core::PixelFormat::RgbResv8BitPerColor,
                PixelFormat::Bgr => mikan_core::PixelFormat::BgrResv8BitPerColor,
                _ => return Err(anyhow!("Unknown pixel format")),
            },
        })
    }

    fn fill_screen(&mut self, frame_buffer: &mut FrameBufferConfig) {
        frame_buffer.buf.fill(0xff);
    }

    fn load_kernel(&mut self, root_dir: &mut Directory) -> Result<usize> {
        let mut file = self.load_file(root_dir, "\\kernel.elf")?;
        let mut buffer = allocate_aligned::<FileInfo>(14);
        let info: &mut FileInfo = file
            .get_info(&mut buffer)
            .map_err(|e| anyhow!("Failed to get information of the file: {:?}", e))?;
        let kernel_size = info.file_size() as usize;

        println!("Kernel size is {} bytes", kernel_size)?;

        let buf = self
            .system_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, kernel_size)
            .map(|ptr| unsafe { core::slice::from_raw_parts_mut(ptr, kernel_size) })
            .map_err(err!("Failed to allocate {} bytes temporary", kernel_size))?;

        file.read(buf)
            .map_err(err!("Failed to read kernel from the file"))?;

        let elf = Elf::from_bytes(buf).map_err(err!("Failed to read ELF file"))?;
        let entry_point = elf.entry_point() as usize;
        let (first, last) = elf
            .program_header_iter()
            .filter(|h| h.ph_type() == ProgramType::LOAD)
            .fold((u64::MAX, 0), |(first, last), h| {
                (min(first, h.vaddr()), max(last, h.vaddr() + h.memsz()))
            });

        self.system_table
            .boot_services()
            .allocate_pages(
                AllocateType::Address(KERNEL_ADDRESS),
                MemoryType::LOADER_DATA,
                ((last - first + 0xfff) / 0x1000) as usize,
            )
            .map_err(err!(
                "Failed to allocate {} bytes at {:#x}",
                kernel_size,
                KERNEL_ADDRESS
            ))?;

        elf.program_header_iter()
            .filter(|h| h.ph_type() == ProgramType::LOAD)
            .for_each(|h| {
                let offset = h.offset() as usize;
                let memory_size = h.memsz() as usize;
                let file_size = h.filesz() as usize;
                let diff = memory_size - file_size;

                unsafe { core::slice::from_raw_parts_mut(h.vaddr() as *mut u8, memory_size) }
                    .copy_from_slice(&buf[offset..offset + file_size]);

                unsafe {
                    core::slice::from_raw_parts_mut(
                        (h.vaddr() as usize + memory_size) as *mut u8,
                        diff,
                    )
                }
                .fill(0);
            });

        println!(
            "Loaded kernel to {:#x} ({} bytes)",
            KERNEL_ADDRESS, kernel_size
        )?;

        Ok(entry_point)
    }

    fn boot(self, entry_point: usize, frame_buffer: FrameBufferConfig) -> Result<()> {
        println!("Booting kernel, exiting boot services")?;

        let mut buf = allocate_aligned::<MemoryDescriptor>(4096);
        self.system_table
            .exit_boot_services(self.handle, &mut buf)
            .map(|_| ())
            .map_err(|_| anyhow!("Could not exit boot services"))?;

        (unsafe { core::mem::transmute::<_, Entrypoint>(entry_point) })(KernelArgs { frame_buffer })
    }

    fn execute(mut self) -> Result<()> {
        println!("Hello, world!")?;

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

        let mut frame_buffer = self.get_frame_buffer()?;

        self.fill_screen(&mut frame_buffer);

        self.load_kernel(&mut root_dir)
            .and_then(|entry_point| self.boot(entry_point, frame_buffer))
    }
}

fn try_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Result<()> {
    uefi_services::init(&mut system_table).map_err(err!("Failed to initialise UEFI services"))?;
    Application::new(handle, system_table).execute()
}

#[entry]
fn main(handle: Handle, system_table: SystemTable<Boot>) -> Status {
    if let Err(e) = try_main(handle, system_table) {
        eprintln!("ERROR: {}", e);
    };

    loop {
        aarch64::instructions::halt();
    }
}
