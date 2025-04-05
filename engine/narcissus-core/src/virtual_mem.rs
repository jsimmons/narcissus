use crate::libc;

#[derive(Clone, Copy, Debug)]
pub enum MapError {
    MapFailed,
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MapError {}

/// Reserve a virtual memory range.
///
/// Size will be rounded up to align with the system's page size.
///
/// The range is valid but inaccessible before calling `virtual_commit`.
#[cold]
#[inline(never)]
pub fn virtual_reserve(size: usize) -> Result<*mut std::ffi::c_void, MapError> {
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

    if ptr == libc::MAP_FAILED || ptr.is_null() {
        Err(MapError::MapFailed)
    } else {
        Ok(ptr)
    }
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
    unsafe {
        let result = libc::mprotect(ptr, size, libc::PROT_READ | libc::PROT_WRITE);
        assert!(result == 0);
    }
}

/// Release a reserved or comitted virtual memory range.
///
/// # Safety
///
/// - Must point to an existing assignment created by [`virtual_reserve`].
/// - `size` must be within range of that reservation.
#[cold]
#[inline(never)]
pub unsafe fn virtual_free(ptr: *mut std::ffi::c_void, size: usize) -> Result<(), MapError> {
    unsafe {
        let result = libc::munmap(ptr, size);
        if result != 0 {
            Err(MapError::MapFailed)
        } else {
            Ok(())
        }
    }
}
