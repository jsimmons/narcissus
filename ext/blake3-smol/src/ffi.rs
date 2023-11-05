
#[allow(dead_code)]
extern "C" {
    pub fn blake3_compress_in_place_avx512(
        cv: *mut u32,
        block: *const u8,
        block_len: u8,
        counter: u64,
        flags: u8,
    );

    pub fn blake3_compress_xof_avx512(
        cv: *const u32,
        block: *const u8,
        block_len: u8,
        counter: u64,
        flags: u8,
        out: *mut u8,
    );

    pub fn blake3_hash_many_avx512(
        inputs: *const *const u8,
        num_inputs: usize,
        blocks: usize,
        key: *const u32,
        counter: u64,
        increment_counter: bool,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: *mut u8,
    );

    pub fn blake3_hash_many_avx2(
        inputs: *const *const u8,
        num_inputs: usize,
        blocks: usize,
        key: *const u32,
        counter: u64,
        increment_counter: bool,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: *mut u8,
    );

    pub fn blake3_compress_in_place_sse41(
        cv: *mut u32,
        block: *const u8,
        block_len: u8,
        counter: u64,
        flags: u8,
    );

    pub fn blake3_compress_xof_sse41(
        cv: *const u32,
        block: *const u8,
        block_len: u8,
        counter: u64,
        flags: u8,
        out: *mut u8,
    );

    pub fn blake3_hash_many_sse41(
        inputs: *const *const u8,
        num_inputs: usize,
        blocks: usize,
        key: *const u32,
        counter: u64,
        increment_counter: bool,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: *mut u8,
    );

    pub fn blake3_compress_in_place_sse2(
        cv: *mut u32,
        block: *const u8,
        block_len: u8,
        counter: u64,
        flags: u8,
    );

    pub fn blake3_compress_xof_sse2(
        cv: *const u32,
        block: *const u8,
        block_len: u8,
        counter: u64,
        flags: u8,
        out: *mut u8,
    );

    pub fn blake3_hash_many_sse2(
        inputs: *const *const u8,
        num_inputs: usize,
        blocks: usize,
        key: *const u32,
        counter: u64,
        increment_counter: bool,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: *mut u8,
    );
}

mod avx512 {
    use std::arch::global_asm;
    #[cfg(target_family = "unix")]
    global_asm!(include_str!("asm/blake3_avx512_x86-64_unix.S"));
    #[cfg(target_family = "windows")]
    global_asm!(include_str!("asm/blake3_avx512_x86-64_windows_gnu.S"));
}

mod avx2 {
    use std::arch::global_asm;
    #[cfg(target_family = "unix")]
    global_asm!(include_str!("asm/blake3_avx2_x86-64_unix.S"));
    #[cfg(target_family = "windows")]
    global_asm!(include_str!("asm/blake3_avx2_x86-64_windows_gnu.S"));
}

mod sse41 {
    use std::arch::global_asm;
    #[cfg(target_family = "unix")]
    global_asm!(include_str!("asm/blake3_sse41_x86-64_unix.S"));
    #[cfg(target_family = "windows")]
    global_asm!(include_str!("asm/blake3_sse41_x86-64_windows_gnu.S"));
}

mod sse2 {
    use std::arch::global_asm;
    #[cfg(target_family = "unix")]
    global_asm!(include_str!("asm/blake3_sse2_x86-64_unix.S"));
    #[cfg(target_family = "windows")]
    global_asm!(include_str!("asm/blake3_sse2_x86-64_windows_gnu.S"));
}
