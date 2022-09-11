#[allow(unconditional_panic)]
const fn illegal_null_in_string() {
    [][0]
}

#[doc(hidden)]
pub const fn validate_cstr_contents(bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\0' {
            illegal_null_in_string();
        }
        i += 1;
    }
}

#[macro_export]
macro_rules! cstr {
    ( $s:literal ) => {{
        $crate::helpers::validate_cstr_contents($s.as_bytes());
        #[allow(unused_unsafe)]
        unsafe {
            std::mem::transmute::<_, &std::ffi::CStr>(concat!($s, "\0"))
        }
    }};
}

#[allow(dead_code)]
pub fn string_from_c_str(c_str: &[i8]) -> String {
    let s = unsafe { std::ffi::CStr::from_ptr(c_str.as_ptr()).to_bytes() };
    String::from_utf8_lossy(s).into_owned()
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    #[test]
    fn test_cstr() {
        assert_eq!(
            cstr!("hello"),
            CStr::from_bytes_with_nul(b"hello\0").unwrap()
        );
    }
}
