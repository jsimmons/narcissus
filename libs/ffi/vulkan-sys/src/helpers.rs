#[doc(hidden)]
pub const fn validate_cstr_contents(bytes: &[u8]) {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\0' {
            panic!("illegal null byte in string");
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
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes())
        }
    }};
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
