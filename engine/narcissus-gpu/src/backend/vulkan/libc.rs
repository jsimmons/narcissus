use std::os::raw::{c_char, c_int, c_void};

pub const RTLD_NOW: c_int = 0x2;
pub const RTLD_LOCAL: c_int = 0;

unsafe extern "C" {
    pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
