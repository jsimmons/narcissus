#[cold]
#[inline(never)]
pub unsafe fn virtual_reserve(size: usize) -> *mut std::ffi::c_void {
    let ptr = libc::mmap(
        std::ptr::null_mut(),
        size,
        libc::PROT_NONE,
        libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
        -1,
        0,
    );

    assert!(ptr != libc::MAP_FAILED && !ptr.is_null());

    ptr
}

#[cold]
#[inline(never)]
pub unsafe fn virtual_commit(ptr: *mut std::ffi::c_void, size: usize) {
    let result = libc::mprotect(ptr, size, libc::PROT_READ | libc::PROT_WRITE);
    assert!(result == 0);
}

#[cold]
#[inline(never)]
pub unsafe fn virtual_free(ptr: *mut std::ffi::c_void, size: usize) {
    let result = libc::munmap(ptr, size);
    assert!(result == 0);
}
