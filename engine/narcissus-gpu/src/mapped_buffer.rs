use std::{marker::PhantomData, ptr::NonNull};

use crate::{Buffer, BufferAddress, BufferArg};

#[cold]
fn overflow() -> ! {
    panic!("overflow")
}

/// Copies the byte representation of T into the given pointer.
///
/// # Panics
///
/// Panics if `len` is insufficient for the object `src` to be placed at the given
/// `offset`
///
/// # Safety
///
/// The memory region from `ptr` through `ptr` + `len` must be valid.
///
/// This function will propagate undefined values from T, for example, padding
/// bytes, so it's vital that no Rust reference to the written memory exists
/// after writing a `T` which contains undefined values.
unsafe fn copy_from_with_offset<T: ?Sized>(ptr: NonNull<u8>, len: usize, offset: usize, src: &T) {
    unsafe {
        let size = std::mem::size_of_val(src);

        let Some(end) = offset.checked_add(size) else {
            overflow()
        };

        if end > len {
            overflow()
        }

        // SAFETY:
        //  * Taking a pointer of `T` as bytes is always valid, even when it contains
        //    padding. So long as we never materialize a reference to those undef bytes
        //    and directly copy through the pointer instead.
        //
        //  * The number of bytes we're reading from src is directly derived from its
        //    size in bytes.
        //
        //  * We check the length of the buffer is sufficient for `size` plus `offset`
        //    bytes above.
        //
        //  * `src` and `dst` cannot overlap because it's not possible to make a
        //    reference to the bytes from the transient buffer.
        let count = size;
        let src = src as *const _ as *const u8;
        let src = src.add(offset);
        let dst = ptr.as_ptr();
        std::ptr::copy_nonoverlapping(src, dst, count)
    }
}

/// Persistent mapped buffer.
///
/// Making sure the buffer is not updated while it is concurrently in use by the
/// GPU is the responsibility of the caller.
pub struct PersistentBuffer<'a> {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) len: usize,
    pub(crate) buffer: Buffer,
    pub(crate) phantom: PhantomData<&'a u8>,
}

impl PersistentBuffer<'_> {
    pub fn to_arg(&self) -> BufferArg {
        BufferArg::Persistent(self)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    pub fn copy_from<T: ?Sized>(&mut self, src: &T) {
        unsafe { copy_from_with_offset(self.ptr, self.len, 0, src) }
    }

    pub fn copy_with_offset<T: ?Sized>(&mut self, offset: usize, src: &T) {
        unsafe { copy_from_with_offset(self.ptr, self.len, offset, src) }
    }
}

/// Transient mapped buffer that is tied to the lifetime of the current frame.
///
/// This buffer will be invalidated and recycled at the end of the frame.
pub struct TransientBuffer<'a> {
    pub(crate) ptr: NonNull<u8>,
    pub(crate) offset: u64,
    pub(crate) len: usize,
    pub(crate) buffer: u64,
    pub(crate) address: BufferAddress<'a>,
}

impl TransientBuffer<'_> {
    pub fn to_arg(&self) -> BufferArg {
        BufferArg::Transient(self)
    }

    pub fn copy_from<T: ?Sized>(&mut self, src: &T) {
        unsafe { copy_from_with_offset(self.ptr, self.len, 0, src) }
    }

    pub fn copy_with_offset<T: ?Sized>(&mut self, offset: usize, src: &T) {
        unsafe { copy_from_with_offset(self.ptr, self.len, offset, src) }
    }
}

pub struct TransientBindGroup<'a> {
    pub(crate) bind_group: u64,
    pub(crate) phantom: PhantomData<&'a ()>,
}
