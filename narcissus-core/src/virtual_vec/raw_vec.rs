use std::cmp;
use std::{
    mem::{align_of, size_of},
    ptr::NonNull,
};

use crate::{page_size, virtual_commit, virtual_free, virtual_reserve};

pub struct VirtualRawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    max_cap: usize,
}

impl<T> VirtualRawVec<T> {
    pub fn new(max_capacity: usize) -> Self {
        assert!(max_capacity != 0);

        let size = size_of::<T>();
        let align = align_of::<T>();
        let page_size = page_size();

        // Allocating memory with virtual alloc for a zst seems a bit of a waste :)
        assert!(size != 0);

        // mmap gaurantees we get page aligned addresses back. So as long as our alignment
        // requirement is less than that, we're all good in the hood.
        assert!(align < page_size);

        let max_capacity_bytes = size.checked_mul(max_capacity).unwrap();

        // Check overflow of rounding operation.
        assert!(max_capacity_bytes <= (std::usize::MAX - (align - 1)));

        let ptr = virtual_reserve(max_capacity_bytes).expect("mapping failed");
        let ptr = unsafe { NonNull::new_unchecked(ptr as *mut T) };

        Self {
            ptr,
            cap: 0,
            max_cap: max_capacity,
        }
    }

    pub fn with_capacity(capacity: usize, max_capacity: usize) -> Self {
        assert!(capacity <= max_capacity);
        let mut vec = Self::new(max_capacity);

        unsafe {
            // we ensure that capacity is less than max_capacity, and the new function above would
            // have paniced if max_capacity * size_of::<T>() overflowed, so we're always safe here.
            let cap_bytes = capacity * size_of::<T>();
            virtual_commit(vec.ptr.as_ptr() as *mut std::ffi::c_void, cap_bytes);
            vec.cap = capacity;
        }

        vec
    }

    #[inline]
    pub fn reserve(&mut self, used_capacity: usize, required_extra_capacity: usize) {
        if self.cap.wrapping_sub(used_capacity) >= required_extra_capacity {
            return;
        }

        self.grow(used_capacity, required_extra_capacity);
    }

    #[cold]
    #[inline(never)]
    pub fn grow(&mut self, used_capacity: usize, required_extra_capacity: usize) {
        unsafe {
            let required_cap = used_capacity.checked_add(required_extra_capacity).unwrap();
            let max_cap = self.max_cap;
            if required_cap > max_cap {
                panic!("max capacity exceeded")
            };

            // cap can never be big enough that this would wrap.
            let double_cap = self.cap * 2;
            let new_cap = cmp::max(required_cap, cmp::min(double_cap, max_cap));

            // This can't overflow because we've already ensured that the new_cap is less than or
            // equal to the the max_cap, and the max_cap has already been checked for overflow in
            // the constructor.
            let new_cap_bytes = new_cap * size_of::<T>();
            virtual_commit(self.ptr.as_ptr() as *mut std::ffi::c_void, new_cap_bytes);

            self.cap = new_cap;
        }
    }

    #[cold]
    #[inline(never)]
    pub fn double(&mut self) {
        unsafe {
            let old_cap = self.cap;
            let min_cap = 1;
            let double_cap = old_cap.wrapping_mul(2);
            let new_cap = cmp::max(double_cap, min_cap);
            let new_cap = cmp::min(new_cap, self.max_cap);
            assert_ne!(old_cap, new_cap);
            let new_cap_bytes = new_cap * size_of::<T>();
            virtual_commit(self.ptr.as_ptr() as *mut std::ffi::c_void, new_cap_bytes);
            self.cap = new_cap;
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline(always)]
    pub fn max_capacity(&self) -> usize {
        self.max_cap
    }

    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Drop for VirtualRawVec<T> {
    fn drop(&mut self) {
        unsafe {
            // The preconditions here that max_cap multiplied by the size won't overflow and
            // that the pointer actually exists and is mapped are all ensured by the constructor.
            virtual_free(
                self.ptr.as_ptr() as *mut std::ffi::c_void,
                self.max_cap * size_of::<T>(),
            )
            .expect("failed to unmap memory");
        }
    }
}
