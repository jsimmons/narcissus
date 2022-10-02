use crate::libc;

/// Reserve a virtual memory range.
///
/// Size will be rounded up to align with the system's page size.
///
/// The range is valid but inaccessible before calling `virtual_commit`.
///
/// # Panics
///
/// Panics if mapping fails.
#[cold]
#[inline(never)]
pub fn virtual_reserve(size: usize) -> *mut std::ffi::c_void {
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    assert!(ptr != libc::MAP_FAILED && !ptr.is_null());

    ptr
}

/// Commit (part of) a previously reserved memory range.
///
/// Marks the range as readable and writable.
///
/// Size will be rounded up to align with the system's page size.
///
/// # Safety
///
/// - Must point to an existing assignment created by [`virtual_reserve`].
/// - size must be within range of that reservation.
///
/// # Panics
///
/// Panics if changing page permissions for the range fails.
#[cold]
#[inline(never)]
pub unsafe fn virtual_commit(ptr: *mut std::ffi::c_void, size: usize) {
    let result = libc::mprotect(ptr, size, libc::PROT_READ | libc::PROT_WRITE);
    assert!(result == 0);
}

/// Release a reserved or comitted virtual memory range.
///
/// # Safety
///
/// - Must point to an existing assignment created by [`virtual_reserve`].
/// - `size` must be within range of that reservation.
///
/// # Panics
///
/// Panics if the range could not be unmapped.
#[cold]
#[inline(never)]
pub unsafe fn virtual_free(ptr: *mut std::ffi::c_void, size: usize) {
    let result = libc::munmap(ptr, size);
    assert!(result == 0);
}
