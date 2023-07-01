use core::ffi::c_void;
use core::mem::size_of;

use num_traits::FromPrimitive;
use xhci::ring::trb::event::{CommandCompletion, PortStatusChange, TransferEvent};
use xhci::ring::trb::Type;

use crate::println;
use crate::usb::memory::Pool;
use crate::usb::Controller;

#[derive(Debug)]
enum TrbPayload<'t> {
    TransferEvent(&'t TransferEvent),
    PortStatusChange(&'t PortStatusChange),
    CommandCompletion(&'t CommandCompletion),
}

#[repr(C, packed)]
#[derive(Debug, Default, Copy, Clone)]
struct Trb {
    _dummy2: u64,
    _dummy: u32,
    bit: u32,
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
}

impl<'r> EventRing<'r> {
    pub(crate) fn new(size: usize, controller: &mut Controller) -> Self {
        let pool = Pool::get();
        let buffer = pool.allocate_slice(size, 64, 64 * 1024);
        buffer.fill(Trb::default());

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

        println!("ERDP: {:04X}", &ring.buffer[0] as *const Trb as u64);
        ring.write_dequeue_pointer(controller, &ring.buffer[0]);

        println!(
            "ERSTBA {:16X}",
            ring.segment_table as *mut [SegmentTableEntry] as *mut c_void as u64
        );
        controller.interrupter().erstba.update_volatile(|r| {
            r.set(ring.segment_table as *mut [SegmentTableEntry] as *mut c_void as u64);
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
        let mut ptr = unsafe { (self.read_dequeue_pointer(controller) as *const Trb).add(1) };
        let segment_begin = self.segment_table[0].base as *const Trb;
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

    fn front(&self, controller: &mut Controller) -> Trb {
        *self.read_dequeue_pointer(controller)
    }

    fn read_dequeue_pointer(&self, controller: &mut Controller) -> &Trb {
        unsafe {
            &*(controller
                .interrupter()
                .erdp
                .read_volatile()
                .event_ring_dequeue_pointer() as *const Trb)
        }
    }

    fn write_dequeue_pointer(&mut self, controller: &mut Controller, trb: *const Trb) {
        let ptr = trb as u64;
        controller.interrupter().erdp.update_volatile(|r| {
            r.set_event_ring_dequeue_pointer(ptr);
        });
    }
}
