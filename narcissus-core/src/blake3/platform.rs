use crate::slice::{array_chunks, array_chunks_mut};

use super::{portable, CVWords, IncrementCounter, BLOCK_LEN};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub const MAX_SIMD_DEGREE: usize = 8;

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
}

impl Platform {
    #[allow(unreachable_code)]
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
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
                super::sse2::compress_in_place(cv, block, block_len, counter, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                super::sse41::compress_in_place(cv, block, block_len, counter, flags)
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
                super::sse2::compress_xof(cv, block, block_len, counter, flags)
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 | Platform::AVX2 => unsafe {
                super::sse41::compress_xof(cv, block, block_len, counter, flags)
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
                super::sse2::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::SSE41 => unsafe {
                super::sse41::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
                )
            },
            // Safe because detect() checked for platform support.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            Platform::AVX2 => unsafe {
                super::avx2::hash_many(
                    inputs,
                    key,
                    counter,
                    increment_counter,
                    flags,
                    flags_start,
                    flags_end,
                    out,
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
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn avx2_detected() -> bool {
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "avx2")]
    {
        return true;
    }

    #[cfg(not(target_feature = "avx2"))]
    return is_x86_feature_detected!("avx2");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn sse41_detected() -> bool {
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "sse4.1")]
    {
        return true;
    }

    #[cfg(not(target_feature = "sse4.1"))]
    return is_x86_feature_detected!("sse4.1");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
#[allow(unreachable_code)]
pub fn sse2_detected() -> bool {
    // Static check, e.g. for building with target-cpu=native.
    #[cfg(target_feature = "sse2")]
    {
        return true;
    }

    #[cfg(not(target_feature = "sse2"))]
    return is_x86_feature_detected!("sse2");
}

#[inline(always)]
pub fn words_from_le_bytes_32(bytes: &[u8; 32]) -> [u32; 8] {
    let mut out = [0; 8];
    for (chunk, word) in array_chunks(bytes).zip(out.iter_mut()) {
        *word = u32::from_le_bytes(*chunk)
    }
    out
}

#[inline(always)]
pub fn words_from_le_bytes_64(bytes: &[u8; 64]) -> [u32; 16] {
    let mut out = [0; 16];
    for (chunk, word) in array_chunks(bytes).zip(out.iter_mut()) {
        *word = u32::from_le_bytes(*chunk)
    }
    out
}

#[inline(always)]
pub fn le_bytes_from_words_32(words: &[u32; 8]) -> [u8; 32] {
    let mut out = [0; 32];
    for (word, chunk) in words.iter().zip(array_chunks_mut(&mut out)) {
        *chunk = word.to_le_bytes();
    }
    out
}

#[inline(always)]
pub fn le_bytes_from_words_64(words: &[u32; 16]) -> [u8; 64] {
    let mut out = [0; 64];
    for (word, chunk) in words.iter().zip(array_chunks_mut(&mut out)) {
        *chunk = word.to_le_bytes();
    }
    out
}
