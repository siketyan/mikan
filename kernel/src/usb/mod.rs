mod memory;
mod ring;

use core::ffi::c_void;
use core::num::NonZeroUsize;
use core::ptr::null_mut;

use accessor::marker::ReadWrite;
use accessor::Mapper;
use once_cell::unsync::OnceCell;
use xhci::extended_capabilities::List;
use xhci::registers::runtime::Interrupter;
use xhci::{ExtendedCapability, Registers};

use crate::println;
use crate::usb::memory::Pool;
use crate::usb::ring::EventRing;

#[derive(Debug, Clone)]
pub(crate) struct MapperImpl;

impl Mapper for MapperImpl {
    unsafe fn map(&mut self, phys_start: usize, _bytes: usize) -> NonZeroUsize {
        NonZeroUsize::new(phys_start).unwrap() // TODO
    }

    fn unmap(&mut self, _virt_start: usize, _bytes: usize) {
        // TODO
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ConfigPhase {
    NotConnected,
    WaitingAddressed,
    ResettingPort,
    EnablingSlot,
    AddressingDevice,
    InitializingDevice,
    ConfiguringEndpoints,
    Configured,
}

pub(crate) struct Port {
    pub(crate) number: usize,
    config_phase: ConfigPhase,
}

impl Port {
    pub(crate) fn is_connected(&self, controller: &Controller) -> bool {
        controller
            .registers
            .port_register_set
            .read_volatile_at(self.number)
            .portsc
            .current_connect_status()
    }

    pub(crate) fn configure(&mut self, controller: &mut Controller) {
        if self.config_phase == ConfigPhase::NotConnected {
            self.reset(controller);
        }
    }

    pub(crate) fn reset(&mut self, controller: &mut Controller) {
        let is_connected = self.is_connected(controller);
        println!("reset: port.is_connected() = {}", is_connected);

        self.config_phase = ConfigPhase::ResettingPort;

        controller
            .registers
            .port_register_set
            .update_volatile_at(self.number, |r| {
                r.portsc.set_port_reset();
                r.portsc.clear_connect_status_change();
            });

        while controller
            .registers
            .port_register_set
            .read_volatile_at(self.number)
            .portsc
            .port_reset()
        {}
    }
}

struct DeviceContext {}

#[derive(Debug)]
struct DeviceManager<'d> {
    max_slots: usize,
    devices: (),
    device_context_pointers: &'d mut [*mut DeviceContext],
}

impl<'d> DeviceManager<'d> {
    fn new(max_slots: usize) -> Self {
        let device_context_pointers = Pool::get().allocate_slice(max_slots + 1, 64, 4096);
        device_context_pointers.fill(null_mut());

        Self {
            max_slots,
            devices: (),
            device_context_pointers,
        }
    }
}

pub(crate) struct Controller<'c> {
    base: usize,
    registers: Registers<MapperImpl>,
    manager: OnceCell<DeviceManager<'c>>,
}

impl<'c> Controller<'c> {
    pub(crate) unsafe fn new(mmio_base: usize) -> Self {
        Self {
            base: mmio_base,
            registers: Registers::new(mmio_base, MapperImpl),
            manager: OnceCell::new(),
        }
    }

    pub(crate) fn initialize(&mut self) {
        self.request_hc_ownership();
        self.reset();

        let capability = &self.registers.capability;
        let max_slots = self.max_slots();

        println!("Max Slots: {}", max_slots);

        let max_scratchpad_buffers = capability
            .hcsparams2
            .read_volatile()
            .max_scratchpad_buffers() as usize;

        if max_scratchpad_buffers > 0 {
            let scratchpad_buffer =
                Pool::get().allocate_slice::<*mut c_void>(max_scratchpad_buffers, 64, 4096);

            scratchpad_buffer.iter_mut().for_each(|b| {
                *b = Pool::get().allocate(4096, 4096, 4096).unwrap();
            });
        }

        let manager = DeviceManager::new(max_slots);
        let ptr = manager.device_context_pointers.as_mut_ptr();

        self.manager.set(manager).unwrap();

        self.registers.operational.dcbaap.update_volatile(|r| {
            r.set(ptr as *mut c_void as u64);
        });
    }

    pub(crate) fn run(&mut self) {
        self.registers.operational.usbcmd.update_volatile(|r| {
            r.set_run_stop();
        });

        while self
            .registers
            .operational
            .usbsts
            .read_volatile()
            .hc_halted()
        {}
    }

    pub(crate) fn interrupter(&mut self) -> Interrupter<MapperImpl, ReadWrite> {
        self.registers.interrupter_register_set.interrupter_mut(0)
    }

    pub(crate) fn ring<'r>(&mut self) -> EventRing<'r> {
        let ring = EventRing::new(32, self);

        self.registers.operational.usbcmd.update_volatile(|r| {
            r.set_interrupter_enable();
        });

        ring
    }

    pub(crate) fn max_slots(&self) -> usize {
        self.registers
            .capability
            .hcsparams1
            .read_volatile()
            .number_of_device_slots() as usize
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Port> + 'static {
        let register_set = &self.registers.port_register_set;
        (0..register_set.len()).map(|i| Port {
            number: i,
            config_phase: ConfigPhase::NotConnected,
        })
    }

    fn reset(&mut self) {
        let operational = &mut self.registers.operational;

        operational.usbcmd.update_volatile(|r| {
            r.clear_interrupter_enable();
            r.clear_host_system_error_enable();
            r.clear_enable_wrap_event();

            // Host controller must be halted before resetting it.
            if !operational.usbsts.read_volatile().hc_halted() {
                r.clear_run_stop();
            }
        });

        // Waits the host controller to be halted.
        while !operational.usbsts.read_volatile().hc_halted() {}

        println!("Resetting the xHC controller...");
        operational.usbcmd.update_volatile(|r| {
            r.set_host_controller_reset();
        });

        while operational.usbcmd.read_volatile().host_controller_reset() {}
        while operational.usbsts.read_volatile().controller_not_ready() {}
    }

    fn request_hc_ownership(&self) {
        let mut extended_capabilities = match unsafe {
            List::new(
                self.base,
                self.registers.capability.hccparams1.read_volatile(),
                MapperImpl,
            )
        } {
            Some(l) => l,
            _ => return,
        };

        let mut legacy_support = match extended_capabilities
            .into_iter()
            .filter_map(|c| c.ok())
            .find_map(|c| match c {
                ExtendedCapability::UsbLegacySupport(c) => Some(c),
                _ => None,
            }) {
            Some(c) => c,
            _ => return,
        };

        if legacy_support
            .usblegsup
            .read_volatile()
            .hc_os_owned_semaphore()
        {
            println!("The OS already owns xHC, skipping");

            return;
        }

        legacy_support.usblegsup.update_volatile(|r| {
            r.set_hc_os_owned_semaphore();
        });

        while !legacy_support
            .usblegsup
            .read_volatile()
            .hc_os_owned_semaphore()
        {}

        println!("Now the OS owns xHC!");
    }
}
