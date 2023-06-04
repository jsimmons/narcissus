use crate::static_assert;

/// Trait that allows explicit integer widening.
pub trait Widen<T> {
    /// Returns `self` "widened" to `T`, panics if the conversion would wrap.
    fn widen(self) -> T;
}

// Would need to further restrict widen cases for 32 bit support.
static_assert!(
    usize::BITS == 64,
    "only supports machines with 64 bit usize"
);

#[cold]
#[inline(never)]
fn widening_failure() {
    panic!("failed to widen type, out of bounds")
}

impl Widen<usize> for u8 {
    #[inline(always)]
    fn widen(self) -> usize {
        self as usize
    }
}

impl Widen<usize> for u16 {
    #[inline(always)]
    fn widen(self) -> usize {
        self as usize
    }
}

impl Widen<usize> for u32 {
    #[inline(always)]
    fn widen(self) -> usize {
        self as usize
    }
}

impl Widen<usize> for u64 {
    #[inline(always)]
    fn widen(self) -> usize {
        self as usize
    }
}

impl Widen<usize> for i8 {
    #[inline(always)]
    fn widen(self) -> usize {
        if self < 0 {
            widening_failure()
        }
        self as usize
    }
}

impl Widen<usize> for i16 {
    #[inline(always)]
    fn widen(self) -> usize {
        if self < 0 {
            widening_failure()
        }
        self as usize
    }
}

impl Widen<usize> for i32 {
    #[inline(always)]
    fn widen(self) -> usize {
        if self < 0 {
            widening_failure()
        }
        self as usize
    }
}

impl Widen<usize> for i64 {
    #[inline(always)]
    fn widen(self) -> usize {
        if self < 0 {
            widening_failure()
        }
        self as usize
    }
}
