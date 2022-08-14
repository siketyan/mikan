use alloc::vec::Vec;
use uefi::data_types::Align;

pub(crate) fn allocate(size: usize) -> Vec<u8> {
    let mut buf = Vec::<u8>::with_capacity(size);

    #[allow(clippy::uninit_vec)]
    unsafe {
        buf.set_len(buf.capacity());
    }

    buf
}

pub(crate) fn allocate_aligned<T>(size: usize) -> Vec<u8>
where
    T: Align + ?Sized,
{
    allocate(size * T::alignment())
}
