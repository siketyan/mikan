use core::mem::{size_of, size_of_val, MaybeUninit};
use core::ops::DerefMut;

use once_cell::unsync::Lazy;

const POOL_SIZE: usize = 4096 * 32;

fn ceil(a: usize, b: usize) -> usize {
    a + b - (a % b)
}

pub(crate) struct Pool {
    buffer: &'static mut [u8; POOL_SIZE],
    cursor: usize,
}

static mut MEMORY: MaybeUninit<[u8; POOL_SIZE]> = MaybeUninit::uninit();
static mut POOL: Lazy<Pool> = Lazy::new(Pool::new);

impl Pool {
    fn new() -> Self {
        let buffer = unsafe { MEMORY.assume_init_mut() };
        Self {
            cursor: buffer.as_ptr() as usize,
            buffer,
        }
    }

    pub(crate) fn get() -> &'static mut Self {
        unsafe { POOL.deref_mut() }
    }

    pub(crate) fn allocate<T>(
        &mut self,
        length: usize,
        alignment: usize,
        boundary: usize,
    ) -> Option<&mut T> {
        if alignment > 0 {
            self.cursor = ceil(self.cursor, alignment);
        }

        if boundary > 0 {
            let boundary = ceil(self.cursor, boundary);
            if boundary < self.cursor + length {
                self.cursor = boundary;
            }
        }

        let ptr = self.buffer.as_ptr() as usize;
        if ptr + size_of_val(self.buffer) < self.cursor + length {
            return None;
        }

        let cursor = self.cursor;
        self.cursor += length;

        Some(unsafe { &mut *(cursor as *mut T) })
    }

    pub(crate) fn allocate_slice<T>(
        &mut self,
        size: usize,
        alignment: usize,
        boundary: usize,
    ) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.allocate::<T>(size * size_of::<T>(), alignment, boundary)
                    .unwrap(),
                size,
            )
        }
    }
}
