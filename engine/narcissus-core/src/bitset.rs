use crate::Widen;

pub trait Bits: Copy + Default {
    fn is_zero(self) -> bool;
    /// Clear the least significant set bit and return its index.
    fn clear_least_significant_set_bit(&mut self) -> u32;
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct BitIter<T, I> {
    base: usize,
    words: I,
    word: T,
}

impl<T, I> BitIter<T, I>
where
    T: Bits,
    I: Iterator<Item = T>,
{
    pub fn new(mut words: I) -> Self {
        let word = words.next().unwrap_or_default();
        Self {
            base: 0,
            words,
            word,
        }
    }
}

impl<T, I> Iterator for BitIter<T, I>
where
    T: Bits,
    I: Iterator<Item = T>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.word.is_zero() {
            self.word = self.words.next()?;
            self.base += std::mem::size_of::<T>() * 8;
        }
        let index = self.word.clear_least_significant_set_bit();
        Some(self.base + index.widen())
    }
}

macro_rules! impl_bits {
    ($t:ty) => {
        impl Bits for $t {
            #[inline(always)]
            fn is_zero(self) -> bool {
                self == 0
            }

            #[inline(always)]
            fn clear_least_significant_set_bit(&mut self) -> u32 {
                let b = *self;
                let t = b & (!b + 1);
                let index = b.trailing_zeros();
                *self ^= t;
                index
            }
        }
    };
}

impl_bits!(u64);
impl_bits!(u32);
impl_bits!(u16);
impl_bits!(u8);

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn all_bits_set() {
        let slice = [u64::MAX; 512];
        let mut i = 0;
        for j in BitIter::new(slice.iter().copied()) {
            assert_eq!(i, j);
            i += 1;
        }
        assert_eq!(i, 512 * 64);
    }

    #[test]
    fn iterate_bits() {
        {
            let bits_iter = BitIter::new(std::iter::once(
                0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_u64,
            ));
            let mut i = 0;
            for index in bits_iter {
                assert_eq!(index, i);
                i += 2;
            }
            assert_eq!(i, 64);
        }

        {
            let bits_iter = BitIter::new(
            std::iter::repeat(
                0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_u64,
            )
            .take(10),
        );
            let mut i = 0;
            for index in bits_iter {
                assert_eq!(index, i);
                i += 2;
            }
            assert_eq!(i, 64 * 10);
        }

        assert_eq!(BitIter::new(std::iter::empty::<u64>()).next(), None);
        assert_eq!(BitIter::new(std::iter::repeat(0_u64).take(10)).next(), None);
    }
}
