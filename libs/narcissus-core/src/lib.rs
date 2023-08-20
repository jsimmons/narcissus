mod arena;
mod bitset;
mod directory;
mod finite;
mod fixed_vec;
mod libc;
pub mod linear_log_binning;
pub mod manual_arc;
mod mutex;
pub mod obj;
mod pool;
pub mod rand;
pub mod raw_window;
mod ref_count;
pub mod slice;
pub mod svg;
mod uuid;
mod virtual_mem;
mod virtual_vec;
mod waiter;
mod widen;

pub use arena::{Arena, HybridArena};
pub use bitset::BitIter;
pub use directory::{cache_dir, config_dir, data_dir, runtime_dir};
pub use finite::{FiniteF32, FiniteF64, NotFiniteError};
pub use fixed_vec::FixedVec;
pub use mutex::Mutex;
pub use pool::{Handle, Pool};
pub use ref_count::{Arc, Rc};
pub use uuid::Uuid;
pub use virtual_mem::{virtual_commit, virtual_free, virtual_reserve};
pub use virtual_vec::{VirtualDeque, VirtualVec};
pub use widen::Widen;

use std::{ffi::CStr, mem::MaybeUninit};

#[macro_export]
macro_rules! static_assert {
    ($cond:expr) => {
        $crate::static_assert!($cond, concat!("assertion failed: ", stringify!($cond)));
    };
    ($cond:expr, $($t:tt)+) => {
        const _: () = {
            if !$cond {
                core::panic!($($t)+)
            }
        };
    };
}

#[macro_export]
macro_rules! thread_token_def {
    ($token_name:ident, $container_name:ident, $max_concurrency:expr) => {
        mod private {
            use std::{cell::UnsafeCell, sync::atomic::AtomicUsize};
            use $crate::PhantomUnsend;
            pub struct $token_name {
                index: usize,
                phantom: PhantomUnsend,
            }

            impl $token_name {
                const MAX_CONCURRENCY: usize = $max_concurrency;
                pub fn new() -> Self {
                    static NEXT_THREAD_INDEX: AtomicUsize = AtomicUsize::new(0);
                    let index =
                        NEXT_THREAD_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    assert!(
                        index < Self::MAX_CONCURRENCY,
                        "number of tokens exceeds max concurrency"
                    );
                    Self {
                        index,
                        phantom: PhantomUnsend {},
                    }
                }
            }

            pub struct $container_name<T> {
                slots: [UnsafeCell<T>; $token_name::MAX_CONCURRENCY],
            }

            impl<T> $container_name<T> {
                pub fn new<F>(mut f: F) -> Self
                where
                    F: FnMut() -> T,
                {
                    Self {
                        slots: std::array::from_fn(|_| UnsafeCell::new(f())),
                    }
                }

                pub fn get<'a>(&self, thread_token: &'a $token_name) -> &'a T {
                    // SAFETY: Safe while `thread_token` cannot be shared between threads, copied or modified?
                    unsafe { &*self.slots[thread_token.index].get() }
                }

                pub fn get_mut<'a>(&self, thread_token: &'a mut $token_name) -> &'a mut T {
                    // SAFETY: Safe while `thread_token` cannot be shared between threads, copied or modified?
                    unsafe { &mut *self.slots[thread_token.index].get() }
                }

                pub fn slots_mut(&mut self) -> &mut [T] {
                    unsafe {
                        std::mem::transmute::<
                            &mut [UnsafeCell<T>; $token_name::MAX_CONCURRENCY],
                            &mut [T; $token_name::MAX_CONCURRENCY],
                        >(&mut self.slots)
                    }
                }
            }
        }
        pub use private::{$container_name, $token_name};
    };
}

#[macro_export]
macro_rules! flags_def {
    ($name:ident) => {
        #[derive(PartialEq, Hash, Debug)]
        pub struct $name(u32);

        impl $name {
            #[inline]
            pub fn from_raw(value: u32) -> Self {
                Self(value)
            }

            #[inline]
            pub fn as_raw(self) -> u32 {
                self.0
            }

            #[inline]
            pub fn intersects(self, rhs: Self) -> bool {
                self.0 & rhs.0 != 0
            }

            #[inline]
            pub fn contains(self, rhs: Self) -> bool {
                self.0 & rhs.0 == rhs.0
            }

            #[inline]
            pub fn cardinality(self) -> u32 {
                self.0.count_ones()
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                Self(self.0)
            }
        }

        impl Copy for $name {}

        impl Default for $name {
            fn default() -> Self {
                Self(0)
            }
        }

        impl Eq for $name {}

        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0
            }
        }

        impl std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0
            }
        }

        impl std::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl std::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0
            }
        }
    };
}

#[macro_export]
macro_rules! include_bytes_align {
    ($align:literal, $path:literal) => {{
        #[repr(align($align))]
        struct AlignedBytes<const LEN: usize>([u8; LEN]);
        &AlignedBytes(*include_bytes!($path)).0
    }};
}

/// Avoid the awful `Default::default()` spam.
#[inline(always)]
pub fn default<T: Default>() -> T {
    T::default()
}

#[inline(never)]
#[cold]
pub fn oom() -> ! {
    panic!("out of memory")
}

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
        $crate::validate_cstr_contents($s.as_bytes());
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes())
        }
    }};
}

/// Constructs a new box with uninitialized contents.
#[inline]
pub fn uninit_box<T>() -> Box<MaybeUninit<T>> {
    let layout = std::alloc::Layout::new::<MaybeUninit<T>>();
    unsafe {
        let ptr = std::mem::transmute::<_, *mut MaybeUninit<T>>(std::alloc::alloc(layout));
        Box::from_raw(ptr)
    }
}

/// Constructs a new box with zeroed contents.
#[inline]
pub fn zeroed_box<T>() -> Box<MaybeUninit<T>> {
    let layout = std::alloc::Layout::new::<MaybeUninit<T>>();
    unsafe {
        let ptr = std::mem::transmute::<_, *mut MaybeUninit<T>>(std::alloc::alloc_zeroed(layout));
        Box::from_raw(ptr)
    }
}

/// Converts `Box<MaybeUninit<T>>` to `Box<T>`
///
/// # Safety
///
/// As with [`MaybeUninit::assume_init`], it is up to the caller to guarantee that the value really
/// is in an initialized state. Calling this when the content is not yet fully initialized causes
/// immediate undefined behavior.
///
/// [`MaybeUninit::assume_init`]: std::mem::MaybeUninit::assume_init
#[inline]
pub unsafe fn box_assume_init<T>(value: Box<MaybeUninit<T>>) -> Box<T> {
    let raw = Box::into_raw(value);
    unsafe { Box::from_raw(raw as *mut T) }
}

/// Negative traits aren't stable yet, so use a dummy PhantomData marker to implement !Send
pub type PhantomUnsend = std::marker::PhantomData<*mut ()>;

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

#[must_use]
pub fn align_offset(x: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    let align_mask = align - 1;
    (x + align_mask) & !align_mask
}

#[must_use]
pub fn is_aligned_to<T>(ptr: *const T, align: usize) -> bool {
    if align == 0 || !align.is_power_of_two() {
        panic!("is_aligned_to: align is not a power-of-two");
    }

    (ptr as usize) & (align - 1) == 0
}

#[must_use]
pub fn is_aligned<T>(ptr: *const T) -> bool {
    is_aligned_to(ptr, std::mem::align_of::<T>())
}

pub fn page_size() -> usize {
    4096
}

/// Returns the multiplicative inverse of the number.
///
/// The multiplicative inverse of a number is a number such that `x * mod_inverse(x) = 1` for any
/// **odd** x.
///
/// # Panics
///
/// Panics in debug mode when passed an even value.
pub fn mod_inverse_u64(value: u64) -> u64 {
    debug_assert!(value & 1 == 1);

    // Jeffrey Hurchalla’s method https://arxiv.org/abs/2204.04342
    let x = value.wrapping_mul(3) ^ 2;
    let y = 1_u64.wrapping_sub(value.wrapping_mul(x));
    let x = x.wrapping_mul(y.wrapping_add(1));
    let y = y.wrapping_mul(y);
    let x = x.wrapping_mul(y.wrapping_add(1));
    let y = y.wrapping_mul(y);
    let x = x.wrapping_mul(y.wrapping_add(1));
    let y = y.wrapping_mul(y);
    x.wrapping_mul(y.wrapping_add(1))
}

/// Returns the multiplicative inverse of the number.
///
/// The multiplicative inverse of a number is a number such that `x * mod_inverse(x) = 1` for any
/// **odd** x.
///
/// # Panics
///
/// Panics in debug mode when passed an even value.
pub fn mod_inverse_u32(value: u32) -> u32 {
    debug_assert!(value & 1 == 1);

    // Jeffrey Hurchalla’s method https://arxiv.org/abs/2204.04342
    let x = value.wrapping_mul(3) ^ 2;
    let y = 1_u32.wrapping_sub(value.wrapping_mul(x));
    let x = x.wrapping_mul(y.wrapping_add(1));
    let y = y.wrapping_mul(y);
    let x = x.wrapping_mul(y.wrapping_add(1));
    let y = y.wrapping_mul(y);
    x.wrapping_mul(y.wrapping_add(1))
}

/// Calculates the full result of a product that would otherwise overflow.
///
/// Returns a tuple containing the high and low parts of the result of `x * y`
///
/// # Example
/// ```
/// use narcissus_core::mul_full_width_u64;
/// let x = 1_000_000_000_000;
/// let y = 2_000_000_000;
/// let (hi, lo) = mul_full_width_u64(x, y);
/// let result = (hi as u128) << 64 | lo as u128;
/// assert_eq!(result, 2_000_000_000_000_000_000_000);
/// ```
#[inline(always)]
pub fn mul_full_width_u64(x: u64, y: u64) -> (u64, u64) {
    let result = x as u128 * y as u128;
    ((result >> 64) as u64, result as u64)
}

/// Calculates the full result of a product that would otherwise overflow.
///
/// Returns a tuple containing the high and low parts of the result of `x * y`
///
/// # Example
/// ```
/// use narcissus_core::mul_full_width_u32;
/// let x = 2_500_000;
/// let y = 2_000;
/// let (hi, lo) = mul_full_width_u32(x, y);
/// let result = (hi as u64) << 32 | lo as u64;
/// assert_eq!(result, 5_000_000_000);
/// ```
#[inline(always)]
pub fn mul_full_width_u32(x: u32, y: u32) -> (u32, u32) {
    let result = x as u64 * y as u64;
    ((result >> 32) as u32, result as u32)
}

/// Calculates the full result of a product that would otherwise overflow.
///
/// Returns a tuple containing the high and low parts of the result of `x * y`
///
/// # Example
/// ```
/// use narcissus_core::mul_full_width_u16;
/// let x = 5_000;
/// let y = 20;
/// let (hi, lo) = mul_full_width_u16(x, y);
/// let result = (hi as u32) << 16 | lo as u32;
/// assert_eq!(result, 100_000);
/// ```
#[inline(always)]
pub fn mul_full_width_u16(x: u16, y: u16) -> (u16, u16) {
    let result = x as u32 * y as u32;
    ((result >> 16) as u16, result as u16)
}

/// Calculates the full result of a product that would otherwise overflow.
///
/// Returns a tuple containing the high and low parts of the result of `x * y`
///
/// # Example
/// ```
/// use narcissus_core::mul_full_width_u8;
/// let x = 100;
/// let y = 10;
/// let (hi, lo) = mul_full_width_u8(x, y);
/// let result = (hi as u16) << 8 | lo as u16;
/// assert_eq!(result, 1_000);
/// ```
#[inline(always)]
pub fn mul_full_width_u8(x: u8, y: u8) -> (u8, u8) {
    let result = x as u16 * y as u16;
    ((result >> 8) as u8, result as u8)
}

/// An error indicating that no nul byte was present.
///
/// A slice used to create a [`CStr`] must contain a nul byte somewhere
/// within the slice.
///
/// This error is created by the [`CStr::from_bytes_until_nul`] method.
///
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FromBytesUntilNulError(());

impl std::fmt::Display for FromBytesUntilNulError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "data provided does not contain a nul")
    }
}

/// Creates a C string wrapper from a byte slice.
///
/// This method will create a `CStr` from any byte slice that contains at
/// least one nul byte. The caller does not need to know or specify where
/// the nul byte is located.
///
/// If the first byte is a nul character, this method will return an
/// empty `CStr`. If multiple nul characters are present, the `CStr` will
/// end at the first one.
///
/// If the slice only has a single nul byte at the end, this method is
/// equivalent to [`CStr::from_bytes_with_nul`].
///
/// # Examples
/// ```
/// use std::ffi::CStr;
/// use narcissus_core::cstr_from_bytes_until_nul;
///
/// let mut buffer = [0u8; 16];
/// unsafe {
///     // Here we might call an unsafe C function that writes a string
///     // into the buffer.
///     let buf_ptr = buffer.as_mut_ptr();
///     buf_ptr.write_bytes(b'A', 8);
/// }
/// // Attempt to extract a C nul-terminated string from the buffer.
/// let c_str = cstr_from_bytes_until_nul(&buffer[..]).unwrap();
/// assert_eq!(c_str.to_str().unwrap(), "AAAAAAAA");
/// ```
pub fn cstr_from_bytes_until_nul(bytes: &[u8]) -> Result<&CStr, FromBytesUntilNulError> {
    let nul_pos = memchr::memchr(0, bytes);
    match nul_pos {
        Some(nul_pos) => {
            let subslice = &bytes[..nul_pos + 1];
            // SAFETY: We know there is a nul byte at nul_pos, so this slice
            // (ending at the nul byte) is a well-formed C string.
            Ok(unsafe { CStr::from_bytes_with_nul_unchecked(subslice) })
        }
        None => Err(FromBytesUntilNulError(())),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::{cstr, mod_inverse_u32, mod_inverse_u64};

    #[test]
    fn test_cstr() {
        assert_eq!(
            cstr!("hello"),
            CStr::from_bytes_with_nul(b"hello\0").unwrap()
        );
    }

    // Test is exhaustive and quite slow in debug mode. So ignore by default.
    #[test]
    #[ignore]
    fn test_mod_inverse_u32_exhaustive() {
        let mut x = 1_u32;
        loop {
            let inv_x = mod_inverse_u32(x);
            if x != inv_x {
                // great success!
            } else if x == 1 && inv_x == 1
                || x == 2147483647 && inv_x == 2147483647
                || x == 2147483649 && inv_x == 2147483649
                || x == 4294967295 && inv_x == 4294967295
            {
                // There are 4 square roots of unity modulo 2^32
            } else {
                assert_ne!(inv_x, x);
            }
            assert_eq!(x.wrapping_mul(inv_x), 1);
            if x == u32::MAX {
                break;
            }
            x += 2;
        }
    }

    #[test]
    fn test_mod_inverse_u64() {
        // Chosen by fair dice roll. (very large dice)
        {
            let x = 16594110198632835723_u64;
            assert_eq!(x.wrapping_mul(mod_inverse_u64(x)), 1);
        }
        {
            let x = 528604400148778217_u64;
            assert_eq!(x.wrapping_mul(mod_inverse_u64(x)), 1);
        }
        {
            let x = 3300434641321711815_u64;
            assert_eq!(x.wrapping_mul(mod_inverse_u64(x)), 1);
        }
        {
            let x = 7154793095758979941_u64;
            assert_eq!(x.wrapping_mul(mod_inverse_u64(x)), 1);
        }
        {
            let x = 8737695847511607165_u64;
            assert_eq!(x.wrapping_mul(mod_inverse_u64(x)), 1);
        }
    }
}
