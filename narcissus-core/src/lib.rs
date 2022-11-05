mod arena;
mod bitset;
mod fixed_vec;
mod image;
mod libc;
pub mod manual_arc;
mod mutex;
pub mod obj;
mod pool;
mod ref_count;
pub mod slice;
mod uuid;
mod virtual_mem;
mod virtual_vec;
mod waiter;

pub use arena::{Arena, HybridArena};
pub use bitset::BitIter;
pub use fixed_vec::FixedVec;
pub use image::Image;
pub use mutex::Mutex;
pub use pool::{Handle, Pool};
pub use ref_count::{Arc, Rc};
pub use uuid::Uuid;
pub use virtual_mem::{virtual_commit, virtual_free, virtual_reserve};
pub use virtual_vec::{VirtualDeque, VirtualVec};

use std::mem::MaybeUninit;

#[macro_export]
macro_rules! static_assert {
    ($cond:expr) => {
        $crate::static_assert!($cond, concat!("assertion failed: ", stringify!($cond)));
    };
    ($cond:expr, $($t:tt)+) => {
        #[forbid(const_err)]
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
            use std::cell::UnsafeCell;
            use std::sync::atomic::AtomicUsize;
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
        pub use private::$container_name;
        pub use private::$token_name;
    };
}

#[macro_export]
macro_rules! flags_def {
    ($name:ident) => {
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

        impl PartialEq for $name {
            fn eq(&self, rhs: &Self) -> bool {
                self.0 == rhs.0
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
        $crate::validate_cstr_contents($s.as_bytes());
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

pub fn uninit_box<T>() -> Box<MaybeUninit<T>> {
    let layout = std::alloc::Layout::new::<MaybeUninit<T>>();
    unsafe {
        let ptr = std::mem::transmute::<_, *mut MaybeUninit<T>>(std::alloc::alloc(layout));
        Box::from_raw(ptr)
    }
}

pub fn zeroed_box<T>() -> Box<MaybeUninit<T>> {
    let layout = std::alloc::Layout::new::<MaybeUninit<T>>();
    unsafe {
        let ptr = std::mem::transmute::<_, *mut MaybeUninit<T>>(std::alloc::alloc_zeroed(layout));
        Box::from_raw(ptr)
    }
}

/// Negative traits aren't stable yet, so use a dummy PhantomData marker to implement !Send
pub type PhantomUnsend = std::marker::PhantomData<*mut ()>;

#[must_use]
pub fn align_offset(x: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (x + align - 1) & !(align - 1)
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

/// Returns the base 2 logarithm of the number, rounded down.
///
/// # Panics
///
/// Panics in debug mode when given a value of zero.
pub fn log2_u32(value: u32) -> u32 {
    debug_assert_ne!(value, 0);
    u32::BITS - 1 - value.leading_zeros()
}

/// Returns the base 2 logarithm of the number, rounded down.
///
/// # Panics
///
/// Panics in debug mode when given a value of zero.
pub fn log2_u64(value: u64) -> u32 {
    debug_assert_ne!(value, 0);
    u64::BITS - 1 - value.leading_zeros()
}

/// Returns the base 2 logarithm of the number, rounded down.
///
/// # Panics
///
/// Panics in debug mode when given a value of zero.
pub fn log2_usize(value: usize) -> u32 {
    debug_assert_ne!(value, 0);
    usize::BITS - 1 - value.leading_zeros()
}

/// Returns the multiplicative inverse of the number.
///
/// The multiplicative inverse of a number is a number such that `x * mod_inverse(x) = 1` for any **odd** x.
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
/// The multiplicative inverse of a number is a number such that `x * mod_inverse(x) = 1` for any **odd** x.
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

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::log2_u32;
    use super::log2_u64;
    use super::log2_usize;

    use super::cstr;
    use super::mod_inverse_u32;
    use super::mod_inverse_u64;

    #[test]
    fn test_cstr() {
        assert_eq!(
            cstr!("hello"),
            CStr::from_bytes_with_nul(b"hello\0").unwrap()
        );
    }

    #[test]
    fn test_log2() {
        let mut x = 0;
        for i in 0..u32::BITS {
            x = (x << 1) | 1;
            assert_eq!(log2_u32(x), i);
        }

        let mut x = 0;
        for i in 0..u64::BITS {
            x = (x << 1) | 1;
            assert_eq!(log2_u64(x), i);
        }

        let mut x = 0;
        for i in 0..usize::BITS {
            x = (x << 1) | 1;
            assert_eq!(log2_usize(x), i);
        }
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
