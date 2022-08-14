use alloc::vec::Vec;
use core::mem::MaybeUninit;
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

pub(crate) fn allocate_uninit<T>(size: usize) -> Vec<MaybeUninit<T>> {
    core::iter::repeat_with(MaybeUninit::<T>::uninit)
        .take(size)
        .collect()
}
