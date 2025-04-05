use crate::libc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Errno(pub i32);

/// Returns the system's equivalent of `errno`
pub fn errno() -> Errno {
    #[cfg(not(target_os = "linux"))]
    const _: () = panic!("unsupported os");

    #[cfg(target_os = "linux")]
    Errno(
        // SAFETY: The pointer returned from errno_location is always a valid pointer to the thread's errno variable.
        unsafe { *libc::errno_location() },
    )
}

#[cfg(target_os = "linux")]
pub mod unix {
    use std::ffi::c_int;

    pub const EPERM: c_int = 1;
    pub const ENOENT: c_int = 2;
    pub const ESRCH: c_int = 3;
    pub const EINTR: c_int = 4;
    pub const EIO: c_int = 5;
    pub const ENXIO: c_int = 6;
    pub const E2BIG: c_int = 7;
    pub const ENOEXEC: c_int = 8;
    pub const EBADF: c_int = 9;
    pub const ECHILD: c_int = 10;
    pub const EAGAIN: c_int = 11;
    pub const ENOMEM: c_int = 12;
    pub const EACCES: c_int = 13;
    pub const EFAULT: c_int = 14;
    pub const ENOTBLK: c_int = 15;
    pub const EBUSY: c_int = 16;
    pub const EEXIST: c_int = 17;
    pub const EXDEV: c_int = 18;
    pub const ENODEV: c_int = 19;
    pub const ENOTDIR: c_int = 20;
    pub const EISDIR: c_int = 21;
    pub const EINVAL: c_int = 22;
    pub const ENFILE: c_int = 23;
    pub const EMFILE: c_int = 24;
    pub const ENOTTY: c_int = 25;
    pub const ETXTBSY: c_int = 26;
    pub const EFBIG: c_int = 27;
    pub const ENOSPC: c_int = 28;
    pub const ESPIPE: c_int = 29;
    pub const EROFS: c_int = 30;
    pub const EMLINK: c_int = 31;
    pub const EPIPE: c_int = 32;
    pub const EDOM: c_int = 33;
    pub const ERANGE: c_int = 34;
}

#[cfg(test)]
mod tests {
    use super::errno;

    #[test]
    fn call_errno() {
        let _ = errno();
    }
}
