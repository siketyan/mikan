use crate::println;
use accessor::marker::ReadWrite;
use core::ffi::c_void;
use core::mem::size_of;
use num_traits::FromPrimitive;
use xhci::registers::runtime::Interrupter;
use xhci::ring::trb::event::{CommandCompletion, PortStatusChange, TransferEvent};
use xhci::ring::trb::Type;

use crate::usb::memory::Pool;
use crate::usb::MapperImpl;

#[derive(Debug)]
enum TrbPayload<'t> {
    TransferEvent(&'t TransferEvent),
    PortStatusChange(&'t PortStatusChange),
    CommandCompletion(&'t CommandCompletion),
}

#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
struct Trb {
    bit: u32,
    _dummy: u32,
    _dummy2: u64,
}

impl Trb {
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

    fn payload(&self) -> TrbPayload {
        match self.ty() {
            Type::TransferEvent => TrbPayload::TransferEvent(self.cast()),
            Type::PortStatusChange => TrbPayload::PortStatusChange(self.cast()),
            Type::CommandCompletion => TrbPayload::CommandCompletion(self.cast()),
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
pub(crate) struct EventRing<'r> {
    buffer: &'r mut [Trb],
    cycle_bit: bool,
    segment_table: &'r mut [SegmentTableEntry],
    interrupter: Interrupter<'r, MapperImpl, ReadWrite>,
}

impl<'r> EventRing<'r> {
    pub(crate) fn new(size: usize, interrupter: Interrupter<'r, MapperImpl, ReadWrite>) -> Self {
        let pool = Pool::get();
        let buffer = pool.allocate_slice(size, 64, 64 * 1024);
        buffer.fill(Trb::default());

        let segment_table = Pool::get().allocate_slice(1, 64, 64 * 1024);
        segment_table.fill(SegmentTableEntry::default());

        let mut ring = Self {
            buffer,
            cycle_bit: true,
            segment_table,
            interrupter,
        };

        ring.interrupter.erstsz.update_volatile(|r| {
            r.set(ring.segment_table.len() as u16);
        });

        ring.write_dequeue_pointer(&ring.buffer[0]);

        ring.interrupter.erstba.update_volatile(|r| {
            r.set(ring.segment_table as *mut [SegmentTableEntry] as *mut c_void as u64);
        });

        ring
    }

    pub(crate) fn process_event(&mut self) {
        if !self.has_front() {
            return;
        }

        println!("{:?}", self.front().payload());

        // match self.front().payload() {
        //     _ => todo!(),
        // }

        self.pop();
    }

    fn pop(&mut self) {
        let mut ptr = unsafe { (self.read_dequeue_pointer() as *const Trb).add(1) };
        let segment_begin = self.segment_table[0].base as *const Trb;
        let segment_end = unsafe { segment_begin.add(self.segment_table[0].size as usize) };
        if ptr == segment_end {
            ptr = segment_end;
            self.cycle_bit = !self.cycle_bit;
        }

        self.write_dequeue_pointer(ptr);
    }

    fn has_front(&self) -> bool {
        self.front().cycle_bit() == self.cycle_bit
    }

    fn front(&self) -> Trb {
        *self.read_dequeue_pointer()
    }

    fn read_dequeue_pointer(&self) -> &Trb {
        unsafe {
            &*(self
                .interrupter
                .erdp
                .read_volatile()
                .event_ring_dequeue_pointer() as *const Trb)
        }
    }

    fn write_dequeue_pointer(&mut self, trb: *const Trb) {
        let ptr = trb as u64;
        self.interrupter.erdp.update_volatile(|r| {
            r.set_event_ring_dequeue_pointer(ptr);
        });
    }
}
