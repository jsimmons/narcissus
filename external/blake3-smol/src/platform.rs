use crate::{portable, CVWords, IncrementCounter, BLOCK_LEN, OUT_LEN};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const MAX_SIMD_DEGREE: usize = 16;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub const MAX_SIMD_DEGREE: usize = 1;

// There are some places where we want a static size that's equal to the
// MAX_SIMD_DEGREE, but also at least 2. Constant contexts aren't currently
// allowed to use cmp::max, so we have to hardcode this additional constant
// value. Get rid of this once cmp::max is a const fn.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const MAX_SIMD_DEGREE_OR_2: usize = MAX_SIMD_DEGREE;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub const MAX_SIMD_DEGREE_OR_2: usize = 2;

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Portable,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE2,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    SSE41,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX2,
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    AVX512,
}

impl Platform {
    #[allow(unreachable_code)]
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if avx512_detected() {
                return Platform::AVX512;
            }
            if avx2_detected() {
                return Platform::AVX2;
            }
            if sse41_detected() {
                return Platform::SSE41;
            }
            if sse2_detected() {
                return Platform::SSE2;
            }
        }
        Platform::Portable
    }

    pub fn simd_degree(&self) -> usize {
        let degree = match self {
            Platform::Portable => 1,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => 4,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => 4,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => 8,
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => 16,
        };
        debug_assert!(degree <= MAX_SIMD_DEGREE);
        degree
    }

    pub fn compress_in_place(
        &self,
        cv: &mut CVWords,
        block: &[u8; BLOCK_LEN],
        block_len: u8,
        counter: u64,
        flags: u8,
    ) {
        match self {
            Platform::Portable => portable::compress_in_place(cv, block, block_len, counter, flags),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => unsafe {
                crate::ffi::blake3_compress_in_place_sse2(
                    cv.as_mut_ptr(),
                    block.as_ptr(),
                    block_len,
                    counter,
                    flags,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                crate::ffi::blake3_compress_in_place_sse41(
                    cv.as_mut_ptr(),
                    block.as_ptr(),
                    block_len,
                    counter,
                    flags,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                crate::ffi::blake3_compress_in_place_avx512(
                    cv.as_mut_ptr(),
                    block.as_ptr(),
                    block_len,
                    counter,
                    flags,
                )
            },
        }
    }

    pub fn compress_xof(
        &self,
        cv: &CVWords,
        block: &[u8; BLOCK_LEN],
        block_len: u8,
        counter: u64,
        flags: u8,
    ) -> [u8; 64] {
        match self {
            Platform::Portable => portable::compress_xof(cv, block, block_len, counter, flags),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => unsafe {
                let mut out = [0u8; 64];
                crate::ffi::blake3_compress_xof_sse2(
                    cv.as_ptr(),
                    block.as_ptr(),
                    block_len,
                    counter,
                    flags,
                    out.as_mut_ptr(),
                );
                out
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                let mut out = [0u8; 64];
                crate::ffi::blake3_compress_xof_sse41(
                    cv.as_ptr(),
                    block.as_ptr(),
                    block_len,
                    counter,
                    flags,
                    out.as_mut_ptr(),
                );
                out
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                let mut out = [0u8; 64];
                crate::ffi::blake3_compress_xof_avx512(
                    cv.as_ptr(),
                    block.as_ptr(),
                    block_len,
                    counter,
                    flags,
                    out.as_mut_ptr(),
                );
                out
            },
        }
    }

    // IMPLEMENTATION NOTE
    // ===================
    // hash_many() applies two optimizations. The critically important
    // optimization is the high-performance parallel SIMD hashing mode,
    // described in detail in the spec. This more than doubles throughput per
    // thread. Another optimization is keeping the state vectors transposed
    // from block to block within a chunk. When state vectors are transposed
    // after every block, there's a small but measurable performance loss.
    // Compressing chunks with a dedicated loop avoids this.

    pub fn hash_many<const N: usize>(
        &self,
        inputs: &[&[u8; N]],
        key: &CVWords,
        counter: u64,
        increment_counter: IncrementCounter,
        flags: u8,
        flags_start: u8,
        flags_end: u8,
        out: &mut [u8],
    ) {
        match self {
            Platform::Portable => portable::hash_many(
                inputs,
                key,
                counter,
                increment_counter,
                flags,
                flags_start,
                flags_end,
                out,
            ),
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE2 => unsafe {
                // The Rust hash_many implementations do bounds checking on the `out`
                // array, but the C implementations don't. Even though this is an unsafe
                // function, assert the bounds here.
                assert!(out.len() >= inputs.len() * OUT_LEN);
                crate::ffi::blake3_hash_many_sse2(
                    inputs.as_ptr() as *const *const u8,
                    inputs.len(),
                    N / BLOCK_LEN,
                    key.as_ptr(),
                    counter,
                    increment_counter.yes(),
                    flags,
                    flags_start,
                    flags_end,
                    out.as_mut_ptr(),
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => unsafe {
                // The Rust hash_many implementations do bounds checking on the `out`
                // array, but the C implementations don't. Even though this is an unsafe
                // function, assert the bounds here.
                assert!(out.len() >= inputs.len() * OUT_LEN);
                crate::ffi::blake3_hash_many_sse41(
                    inputs.as_ptr() as *const *const u8,
                    inputs.len(),
                    N / BLOCK_LEN,
                    key.as_ptr(),
                    counter,
                    increment_counter.yes(),
                    flags,
                    flags_start,
                    flags_end,
                    out.as_mut_ptr(),
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => unsafe {
                // The Rust hash_many implementations do bounds checking on the `out`
                // array, but the C implementations don't. Even though this is an unsafe
                // function, assert the bounds here.
                assert!(out.len() >= inputs.len() * OUT_LEN);
                crate::ffi::blake3_hash_many_avx2(
                    inputs.as_ptr() as *const *const u8,
                    inputs.len(),
                    N / BLOCK_LEN,
                    key.as_ptr(),
                    counter,
                    increment_counter.yes(),
                    flags,
                    flags_start,
                    flags_end,
                    out.as_mut_ptr(),
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX512 => unsafe {
                // The Rust hash_many implementations do bounds checking on the `out`
                // array, but the C implementations don't. Even though this is an unsafe
                // function, assert the bounds here.
                assert!(out.len() >= inputs.len() * OUT_LEN);
                crate::ffi::blake3_hash_many_avx512(
                    inputs.as_ptr() as *const *const u8,
                    inputs.len(),
                    N / BLOCK_LEN,
                    key.as_ptr(),
                    counter,
                    increment_counter.yes(),
                    flags,
                    flags_start,
                    flags_end,
                    out.as_mut_ptr(),
                )
            },
        }
    }

    // Explicit platform constructors, for benchmarks.

    pub fn portable() -> Self {
        Self::Portable
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sse2() -> Option<Self> {
        if sse2_detected() {
            Some(Self::SSE2)
        } else {
            None
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sse41() -> Option<Self> {
        if sse41_detected() {
            Some(Self::SSE41)
        } else {
            None
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn avx2() -> Option<Self> {
        if avx2_detected() {
            Some(Self::AVX2)
        } else {
            None
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn avx512() -> Option<Self> {
        if avx512_detected() {
            Some(Self::AVX512)
        } else {
            None
        }
    }
}

// Note that AVX-512 is divided into multiple featuresets, and we use two of
// them, F and VL.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn avx512_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_avx512") {
        return false;
    }

    is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512vl")
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn avx2_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_avx2") {
        return false;
    }

    is_x86_feature_detected!("avx2")
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn sse41_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_sse41") {
        return false;
    }

    is_x86_feature_detected!("sse4.1")
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
#[allow(unreachable_code)]
pub fn sse2_detected() -> bool {
    // A testing-only short-circuit.
    if cfg!(feature = "no_sse2") {
        return false;
    }

    is_x86_feature_detected!("sse2")
}

#[inline(always)]
pub fn words_from_le_bytes_32(bytes: &[u8; 32]) -> [u32; 8] {
    // SAFETY: converting [u8; _] to [[u8; _]; _]  of matching size is always valid.
    let words: [[u8; 4]; 8] = unsafe { std::mem::transmute(*bytes) };
    [
        u32::from_le_bytes(words[0]),
        u32::from_le_bytes(words[1]),
        u32::from_le_bytes(words[2]),
        u32::from_le_bytes(words[3]),
        u32::from_le_bytes(words[4]),
        u32::from_le_bytes(words[5]),
        u32::from_le_bytes(words[6]),
        u32::from_le_bytes(words[7]),
    ]
}

#[inline(always)]
pub fn words_from_le_bytes_64(bytes: &[u8; 64]) -> [u32; 16] {
    // SAFETY: converting [u8; _] to [[u8; _]; _]  of matching size is always valid.
    let words: [[u8; 4]; 16] = unsafe { std::mem::transmute(*bytes) };
    [
        u32::from_le_bytes(words[0]),
        u32::from_le_bytes(words[1]),
        u32::from_le_bytes(words[2]),
        u32::from_le_bytes(words[3]),
        u32::from_le_bytes(words[4]),
        u32::from_le_bytes(words[5]),
        u32::from_le_bytes(words[6]),
        u32::from_le_bytes(words[7]),
        u32::from_le_bytes(words[8]),
        u32::from_le_bytes(words[9]),
        u32::from_le_bytes(words[10]),
        u32::from_le_bytes(words[11]),
        u32::from_le_bytes(words[12]),
        u32::from_le_bytes(words[13]),
        u32::from_le_bytes(words[14]),
        u32::from_le_bytes(words[15]),
    ]
}

#[inline(always)]
pub fn le_bytes_from_words_32(words: &[u32; 8]) -> [u8; 32] {
    let out = [
        words[0].to_le_bytes(),
        words[1].to_le_bytes(),
        words[2].to_le_bytes(),
        words[3].to_le_bytes(),
        words[4].to_le_bytes(),
        words[5].to_le_bytes(),
        words[6].to_le_bytes(),
        words[7].to_le_bytes(),
    ];
    // SAFETY: converting [[u8; _]; _] to [u8; _] of matching size is always valid.
    unsafe { std::mem::transmute(out) }
}

#[inline(always)]
pub fn le_bytes_from_words_64(words: &[u32; 16]) -> [u8; 64] {
    let out = [
        words[0].to_le_bytes(),
        words[1].to_le_bytes(),
        words[2].to_le_bytes(),
        words[3].to_le_bytes(),
        words[4].to_le_bytes(),
        words[5].to_le_bytes(),
        words[6].to_le_bytes(),
        words[7].to_le_bytes(),
        words[8].to_le_bytes(),
        words[9].to_le_bytes(),
        words[10].to_le_bytes(),
        words[11].to_le_bytes(),
        words[12].to_le_bytes(),
        words[13].to_le_bytes(),
        words[14].to_le_bytes(),
        words[15].to_le_bytes(),
    ];
    // SAFETY: converting [[u8; _]; _] to [u8; _] of matching size is always valid.
    unsafe { std::mem::transmute(out) }
}
