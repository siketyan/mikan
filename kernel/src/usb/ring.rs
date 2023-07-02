use core::ffi::c_void;
use core::mem::size_of;

use num_traits::FromPrimitive;
use xhci::ring::trb::event::{CommandCompletion, PortStatusChange, TransferEvent};
use xhci::ring::trb::{Link, Type};

use crate::println;
use crate::usb::memory::Pool;
use crate::usb::Controller;

#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct Trb {
    data: [u32; 4],
}

#[derive(Debug)]
enum EventTrbPayload<'t> {
    TransferEvent(&'t TransferEvent),
    PortStatusChange(&'t PortStatusChange),
    CommandCompletion(&'t CommandCompletion),
}

#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
struct EventTrb {
    _dummy2: u64,
    _dummy: u32,
    bit: u32,
}

impl EventTrb {
    fn cycle_bit(&self) -> bool {
        (self.bit & 1 << 0) != 0
    }

    fn ty(&self) -> Type {
        Type::from_u32(self.bit & 0xf000 >> 3).unwrap()
    }

    fn cast<T>(&self) -> &T {
        assert_eq!(size_of::<Self>(), size_of::<T>());
        unsafe { &*(self as *const Self as *const T) }
    }

    fn payload(&self) -> EventTrbPayload {
        match self.ty() {
            Type::TransferEvent => EventTrbPayload::TransferEvent(self.cast()),
            Type::PortStatusChange => EventTrbPayload::PortStatusChange(self.cast()),
            Type::CommandCompletion => EventTrbPayload::CommandCompletion(self.cast()),
            _ => unimplemented!(),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone)]
struct SegmentTableEntry {
    base: u64,
    size: u16,
    _dummy1: u16,
    _dummy2: u32,
}

#[derive(Debug)]
pub(crate) struct Ring<'r> {
    buffer: &'r mut [Trb],
    cycle_bit: bool,
    cursor: usize,
}

impl<'r> Ring<'r> {
    pub(crate) fn new(size: usize, controller: &mut Controller) -> Self {
        let pool = Pool::get();
        let buffer = pool.allocate_slice(size, 64, 64 * 1024);
        buffer.fill(Trb::default());

        let ring = Self {
            buffer,
            cycle_bit: true,
            cursor: 0,
        };

        controller.registers.operational.crcr.update_volatile(|r| {
            r.set_ring_cycle_state();
            r.set_command_ring_pointer(ring.buffer.as_mut_ptr() as u64);
        });

        ring
    }

    pub(crate) fn copy_to_last(&mut self, data: [u32; 4]) {
        self.buffer[self.cursor].data = data;
        self.buffer[self.cursor].data[3] = (data[3] & 0xfffffffe) | self.cycle_bit as u32;
    }

    pub(crate) fn push(&mut self, data: [u32; 4]) -> *const Trb {
        self.copy_to_last(data);
        self.cursor += 1;

        let ptr = &self.buffer[self.cursor] as *const Trb;
        if self.cursor == self.buffer.len() - 1 {
            let mut link = Link::new();
            link.set_ring_segment_pointer(self.buffer.as_mut_ptr() as u64);
            link.set_toggle_cycle();
            self.copy_to_last(link.into_raw());

            self.cursor = 0;
            self.cycle_bit = !self.cycle_bit;
        }

        ptr
    }
}

#[derive(Debug)]
pub(crate) struct EventRing<'r> {
    buffer: &'r mut [EventTrb],
    cycle_bit: bool,
    segment_table: &'r mut [SegmentTableEntry],
}

impl<'r> EventRing<'r> {
    pub(crate) fn new(size: usize, controller: &mut Controller) -> Self {
        let pool = Pool::get();
        let buffer = pool.allocate_slice(size, 64, 64 * 1024);
        buffer.fill(EventTrb::default());

        let segment_table = Pool::get().allocate_slice(1, 64, 64 * 1024);
        segment_table.fill(SegmentTableEntry::default());

        let mut ring = Self {
            buffer,
            cycle_bit: true,
            segment_table,
        };

        ring.segment_table[0].base = ring.buffer.as_mut_ptr() as u64;
        ring.segment_table[0].size = ring.segment_table.len() as u16;

        println!("ERSTSZ: {:04X}", ring.segment_table.len() as u16);
        controller.interrupter().erstsz.update_volatile(|r| {
            r.set(ring.segment_table.len() as u16);
        });

        println!("ERDP: {:04X}", &ring.buffer[0] as *const EventTrb as u64);
        ring.write_dequeue_pointer(controller, &ring.buffer[0]);

        println!("ERSTBA {:16X}", ring.segment_table.as_mut_ptr() as u64);
        controller.interrupter().erstba.update_volatile(|r| {
            r.set(ring.segment_table.as_mut_ptr() as u64);
        });

        controller.interrupter().iman.update_volatile(|r| {
            r.clear_interrupt_pending();
            r.set_interrupt_enable();
        });

        ring
    }

    pub(crate) fn process_event(&mut self, controller: &mut Controller) {
        if !self.has_front(controller) {
            return;
        }

        println!("{:?}", self.front(controller).payload());

        // match self.front().payload() {
        //     _ => todo!(),
        // }

        self.pop(controller);
    }

    fn pop(&mut self, controller: &mut Controller) {
        let mut ptr = unsafe { (self.read_dequeue_pointer(controller) as *const EventTrb).add(1) };
        let segment_begin = self.segment_table[0].base as *const EventTrb;
        let segment_end = unsafe { segment_begin.add(self.segment_table[0].size as usize) };
        if ptr == segment_end {
            ptr = segment_end;
            self.cycle_bit = !self.cycle_bit;
        }

        self.write_dequeue_pointer(controller, ptr);
    }

    fn has_front(&self, controller: &mut Controller) -> bool {
        self.front(controller).cycle_bit() == self.cycle_bit
    }

    fn front(&self, controller: &mut Controller) -> EventTrb {
        let ptr = self.read_dequeue_pointer(controller);

        return *ptr;
    }

    fn read_dequeue_pointer(&self, controller: &mut Controller) -> &EventTrb {
        unsafe {
            &*(controller
                .interrupter()
                .erdp
                .read_volatile()
                .event_ring_dequeue_pointer() as *const EventTrb)
        }
    }

    fn write_dequeue_pointer(&mut self, controller: &mut Controller, trb: *const EventTrb) {
        let ptr = trb as u64;
        controller.interrupter().erdp.update_volatile(|r| {
            r.set_event_ring_dequeue_pointer(ptr);
        });
    }
}
