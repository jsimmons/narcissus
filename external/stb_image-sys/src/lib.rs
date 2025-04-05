use std::os::raw::{c_char, c_float, c_int, c_uchar, c_void};

mod libc {
    pub enum File {}
    impl Copy for File {}
    impl Clone for File {
        fn clone(&self) -> File {
            *self
        }
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct stbi_io_callbacks {
    /// fill 'data' with 'size' bytes.  return number of bytes actually read
    pub read: extern "C" fn(user: *mut c_void, data: *mut c_char, size: c_int) -> i32,
    /// skip the next 'n' bytes, or 'unget' the last -n bytes if negative
    pub skip: extern "C" fn(user: *mut c_void, n: c_int),
    /// returns nonzero if we are at end of file/data
    pub eof: extern "C" fn(user: *mut c_void) -> i32,
}

unsafe extern "C" {
    pub fn stbi_load_from_memory(
        buffer: *const c_uchar,
        len: c_int,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_uchar;

    pub fn stbi_load(
        filename: *const c_char,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_uchar;

    pub fn stbi_load_from_file(
        f: *mut libc::File,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_uchar;

    pub fn stbi_load_from_callbacks(
        clbk: &stbi_io_callbacks,
        user: *mut c_void,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_uchar;

    pub fn stbi_loadf_from_memory(
        buffer: *const c_uchar,
        len: c_int,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_float;

    pub fn stbi_loadf(
        filename: *const c_char,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_float;

    pub fn stbi_loadf_from_file(
        f: *mut libc::File,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_float;

    pub fn stbi_loadf_from_callbacks(
        clbk: &stbi_io_callbacks,
        user: *mut c_void,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
        req_comp: c_int,
    ) -> *mut c_float;

    pub fn stbi_hdr_to_ldr_gamma(gamma: c_float);

    pub fn stbi_hdr_to_ldr_scale(scale: c_float);

    pub fn stbi_ldr_to_hdr_gamma(gamma: c_float);

    pub fn stbi_ldr_to_hdr_scale(scale: c_float);

    pub fn stbi_is_hdr_from_callbacks(clbk: &stbi_io_callbacks, user: *mut c_void) -> c_int;

    pub fn stbi_is_hdr_from_memory(buffer: *const c_uchar, len: c_int) -> c_int;

    pub fn stbi_is_hdr(filename: *const c_char) -> c_int;

    pub fn stbi_is_hdr_from_file(f: *mut libc::File) -> c_int;

    pub fn stbi_failure_reason() -> *const c_char;

    pub fn stbi_image_free(retval_from_stbi_load: *mut c_void);

    pub fn stbi_info_from_memory(
        buffer: *const c_uchar,
        len: c_int,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
    ) -> c_int;

    pub fn stbi_info_from_callbacks(
        clbk: &stbi_io_callbacks,
        user: *mut c_void,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
    ) -> c_int;

    pub fn stbi_info(
        filename: *const c_char,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
    ) -> c_int;

    pub fn stbi_info_from_file(
        f: *mut libc::File,
        x: &mut c_int,
        y: &mut c_int,
        comp: &mut c_int,
    ) -> c_int;

    pub fn stbi_set_unpremultiply_on_load(flag_true_if_should_unpremultiply: c_int);

    pub fn stbi_convert_iphone_png_to_rgb(flag_true_if_should_convert: c_int);

    pub fn stbi_zlib_decode_malloc_guesssize(
        buffer: *const c_char,
        len: c_int,
        initial_size: c_int,
        outlen: &mut c_int,
    ) -> *mut c_char;

    pub fn stbi_zlib_decode_malloc(
        buffer: *const c_char,
        len: c_int,
        outlen: &mut c_int,
    ) -> *mut c_char;

    pub fn stbi_zlib_decode_buffer(
        obuffer: *const c_char,
        olen: c_int,
        ibuffer: *const c_char,
        ilen: c_int,
    ) -> c_int;

    pub fn stbi_zlib_decode_noheader_malloc(
        buffer: *const c_char,
        len: c_int,
        outlen: &mut c_int,
    ) -> *mut c_char;

    pub fn stbi_zlib_decode_noheader_buffer(
        obuffer: *mut c_char,
        olen: c_int,
        ibuffer: *const c_char,
        ilen: c_int,
    ) -> c_int;

    pub fn stbi_set_flip_vertically_on_load(flag_true_if_should_flip: i32);

}
