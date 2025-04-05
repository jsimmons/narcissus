use crate::{Widen, mul_full_width_u64};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pcg64 {
    state: u128,
}

impl Pcg64 {
    pub const fn new() -> Self {
        Self {
            state: 0x979c9a98d84620057d3e9cb6cfe0549b,
        }
    }

    pub fn with_seed(seed: u128) -> Self {
        let mut rng = Self { state: 0 };
        let _ = rng.next_u64();
        rng.state = rng.state.wrapping_add(seed);
        let _ = rng.next_u64();
        rng
    }

    /// Generates a uniformly distributed random number in the range `0..2^64`.
    #[inline]
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(25492979953554139244865540595714422341)
            .wrapping_add(63641362238467930051442695040888963407);
        ((old_state >> 64) ^ old_state).rotate_right((old_state >> 122) as u32) as u64
    }

    /// Generates a uniformly distributed random number in the range
    /// `0..upper_bound`
    ///
    /// Always draws two 64 bit words from the PRNG.
    ///
    /// Based on <https://github.com/apple/swift/pull/39143/commits/87b3f607042e653a42b505442cc803ec20319c1c>
    #[inline]
    #[must_use]
    pub fn next_bound_u64(&mut self, upper_bound: u64) -> u64 {
        let (result, fraction) = mul_full_width_u64(upper_bound, self.next_u64());
        let (hi, _) = mul_full_width_u64(upper_bound, self.next_u64());
        let (_, carry) = fraction.overflowing_add(hi);
        result + carry as u64
    }

    /// Generates a uniformly distributed random number in the range
    /// `0..upper_bound`
    ///
    /// Always draws two 64 bit words from the PRNG.
    ///
    /// Based on <https://github.com/apple/swift/pull/39143/commits/87b3f607042e653a42b505442cc803ec20319c1c>
    #[inline]
    #[must_use]
    pub fn next_bound_usize(&mut self, upper_bound: usize) -> usize {
        let upper_bound = upper_bound as u64;
        let (result, fraction) = mul_full_width_u64(upper_bound, self.next_u64());
        let (hi, _) = mul_full_width_u64(upper_bound, self.next_u64());
        let (_, carry) = fraction.overflowing_add(hi);
        (result + carry as u64).widen()
    }

    /// Generates a uniformly distributed random f32 in the range `0.0..1.0`
    ///
    /// Always draws one 64 bit word from the PRNG.
    #[inline]
    #[must_use]
    pub fn next_f32(&mut self) -> f32 {
        let value = (self.next_u64() >> (64 - 24)) as f32;
        value * 5.960_464_5e-8 // 0x1p-24
    }

    /// Generates a uniformly distributed random f64 in the range `0.0..1.0`
    ///
    /// Always draws one 64 bit word from the PRNG.
    #[inline]
    #[must_use]
    pub fn next_f64(&mut self) -> f64 {
        let value = (self.next_u64() >> (64 - 53)) as f64;
        value * 1.110_223_024_625_156_5e-16 // 0x1p-53
    }

    /// Generates a uniformly distributed random f32 in the range `-1.0..1.0`
    ///
    /// Always draws one 64 bit word from the PRNG.
    #[inline]
    #[must_use]
    pub fn next_f32_s(&mut self) -> f32 {
        let value = (self.next_u64() as i64 >> (64 - 25)) as f32;
        value * 5.960_464_5e-8 // 0x1p-24
    }

    /// Generates a uniformly distributed random f64 in the range `-1.0..1.0`
    ///
    /// Always draws one 64 bit word from the PRNG.
    #[inline]
    #[must_use]
    pub fn next_f64_s(&mut self) -> f64 {
        let value = (self.next_u64() as i64 >> (64 - 54)) as f64;
        value * 1.110_223_024_625_156_5e-16 // 0x1p-53
    }

    /// Generate a uniformly distributed point on the unit disc using rejection
    /// sampling.
    ///
    /// Returns a tuple containing the dot product of the point with itself, as
    /// well as the point.
    ///
    /// (p.p, [px, py])
    ///
    /// # Notes
    ///
    /// Uniform point on unit disc by Marc B. Reynolds:
    /// <https://marc-b-reynolds.github.io/distribution/2016/11/28/Uniform.html>
    pub fn next_uniform_unit_disc_f32(&mut self) -> (f32, [f32; 2]) {
        let mut x;
        let mut y;
        let mut d;
        loop {
            x = self.next_f32_s();
            y = self.next_f32_s();
            d = x * x + y * y;
            if d < 1.0 {
                break;
            }
        }
        (d, [x, y])
    }

    /// Generate a uniformly distributed point on the unit circle.
    ///
    /// # Notes
    ///
    /// Uniform point on unit circle by Marc B. Reynolds:
    /// <https://marc-b-reynolds.github.io/distribution/2016/11/28/Uniform.html>
    pub fn next_uniform_unit_circle_f32(&mut self) -> [f32; 2] {
        const BIAS: f32 = 1.0 / 68719476736.0; // 0x1p-36
        let (d, [x, y]) = self.next_uniform_unit_disc_f32();
        let s = (d + BIAS * BIAS).sqrt().recip();
        let x = x + BIAS;
        [x * s, y * s]
    }

    /// Randomly select an an element from `slice` with uniform probability.
    ///
    /// Always draws two 64 bit words from the PRNG.
    pub fn select<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            slice.get(self.next_bound_usize(slice.len()))
        }
    }

    /// Randomly select an an element from `slice` with uniform probability.
    ///
    /// Always draws two 64 bit words from the PRNG.
    pub fn select_mut<'a, T>(&mut self, slice: &'a mut [T]) -> Option<&'a mut T> {
        if slice.is_empty() {
            None
        } else {
            slice.get_mut(self.next_bound_usize(slice.len()))
        }
    }

    /// Randomly select an an element from `array` with uniform probability.
    ///
    /// Always draws two 64 bit words from the PRNG.
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0
    pub fn array_select<'a, T, const N: usize>(&mut self, array: &'a [T; N]) -> &'a T {
        &array[self.next_bound_usize(N)]
    }

    /// Randomly select an an element from `array` with uniform probability.
    ///
    /// Always draws two 64 bit words from the PRNG.
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0
    pub fn array_select_mut<'a, T, const N: usize>(&mut self, array: &'a mut [T; N]) -> &'a mut T {
        &mut array[self.next_bound_usize(N)]
    }

    /// Shuffle the elements in `slice` in-place.
    ///
    /// Note that as `Pcg64` is initialized with a 128 bit seed, it's only possible
    /// to generate `2^128` permutations. This means for slices larger than 34
    /// elements, this function can no longer produce all possible permutations.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        if !slice.is_empty() {
            let mut i = slice.len() - 1;
            while i >= 1 {
                let j = self.next_bound_u64((i + 1) as u64) as usize;
                slice.swap(i, j);
                i -= 1;
            }
        }
    }
}

impl Default for Pcg64 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn pcg64_default_sequence() {
        let mut rng = Pcg64::new();
        assert_eq!(rng.next_u64(), 14322063641855463473);
        assert_eq!(rng.next_u64(), 14211086133763074855);
        assert_eq!(rng.next_u64(), 2051302165745047857);
        assert_eq!(rng.next_u64(), 11538586989805838516);
        assert_eq!(rng.next_u64(), 486667062142511543);
    }

    #[test]
    fn pcg64_bounded_random() {
        let mut rng = Pcg64::new();
        assert_eq!(rng.next_bound_u64(1_000), 776);
        assert_eq!(rng.next_bound_u64(1_000), 111);
        assert_eq!(rng.next_bound_u64(1_000), 26);
        assert_eq!(rng.next_bound_u64(1_000), 197);
        assert_eq!(rng.next_bound_u64(10), 1);
        assert_eq!(rng.next_bound_u64(10), 0);
        assert_eq!(rng.next_bound_u64(10), 2);
        assert_eq!(rng.next_bound_u64(10), 9);
        assert_eq!(rng.next_bound_u64(999_999), 254_235);
        assert_eq!(rng.next_bound_u64(999_999), 504_115);
        assert_eq!(rng.next_bound_u64(0), 0);
        assert_eq!(rng.next_bound_u64(0), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(1), 0);
        assert_eq!(rng.next_bound_u64(2), 1);
        assert_eq!(rng.next_bound_u64(2), 1);
        assert_eq!(rng.next_bound_u64(2), 1);
        assert_eq!(rng.next_bound_u64(2), 0);
    }

    #[test]
    fn shuffle_generates_all_permutations() {
        let mut array: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut permutations = HashSet::new();
        let mut rng = Pcg64::new();
        // 8P8 = 40_320 = number of possible permutations of 8 elements.
        while permutations.len() != 40_320 {
            rng.shuffle(&mut array);
            permutations.insert(u64::from_le_bytes(array));
        }
    }

    #[test]
    fn shuffle_empty_slice() {
        let slice: &mut [u8] = &mut [];
        let mut rng = Pcg64::new();
        rng.shuffle(slice)
    }

    #[test]
    fn select_visits_all_elements() {
        let array = &[0, 1, 2, 3, 4, 5, 6, 7];
        let mut selected = HashSet::<u8>::from_iter(array.iter().copied());
        let mut rng = Pcg64::new();
        while !selected.is_empty() {
            selected.remove(rng.select(array).unwrap());
        }
    }

    #[test]
    fn select_empty_slice() {
        let slice: &mut [u8] = &mut [];
        let mut rng = Pcg64::new();
        assert_eq!(rng.select(slice), None);
    }

    #[test]
    fn uniform_f32() {
        let mut rng = Pcg64::new();
        for _ in 0..100_000 {
            let x = rng.next_f32();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn uniform_f64() {
        let mut rng = Pcg64::new();
        for _ in 0..100_000 {
            let x = rng.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn uniform_f32_s() {
        let mut rng = Pcg64::new();
        for _ in 0..100_000 {
            let x = rng.next_f32_s();
            assert!((-1.0..1.0).contains(&x));
        }
    }

    #[test]
    fn uniform_f64_s() {
        let mut rng = Pcg64::new();
        for _ in 0..100_000 {
            let x = rng.next_f64_s();
            assert!((-1.0..1.0).contains(&x));
        }
    }
}
