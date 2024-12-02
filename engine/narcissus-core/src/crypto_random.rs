use std::ffi::c_void;

use crate::{errno, libc};

/// Fill `bytes` with output from the system's cryptographically secure PRNG.
///
/// Linux
/// ---
///
/// Utilizes libc's `getrandom` API in blocking mode.
pub fn fill_random(bytes: &mut [u8]) {
    #[cfg(not(target_os = "linux"))]
    const _: () = panic!("unsupported os");

    #[cfg(target_os = "linux")]
    unsafe {
        loop {
            let res = libc::getrandom(bytes.as_mut_ptr() as *mut c_void, bytes.len(), 0);
            if res == bytes.len() as isize {
                break;
            }

            if res < 0 && errno::errno().0 != errno::unix::EINTR {
                panic!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fill_random;

    #[test]
    fn generate_random_small() {
        let mut data_0 = [0; 256];
        let mut data_1 = [0; 256];

        fill_random(&mut data_0);
        fill_random(&mut data_1);

        assert!(!data_0.iter().all(|&byte| byte == 0));
        assert!(!data_1.iter().all(|&byte| byte == 0));
        assert_ne!(data_0, data_1);
    }

    #[test]
    fn generate_random_large() {
        const MIB: usize = 1024 * 1024;
        let mut data_0 = vec![0; 64 * MIB];
        let mut data_1 = vec![0; 64 * MIB];

        fill_random(&mut data_0);
        fill_random(&mut data_1);

        assert!(!data_0.iter().all(|&byte| byte == 0));
        assert!(!data_1.iter().all(|&byte| byte == 0));
        assert_ne!(data_0, data_1);
    }
}
