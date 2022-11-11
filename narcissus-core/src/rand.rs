use crate::mul_full_width_u64;

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
        rng.state += seed;
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

    /// Generates a uniformly distributed random number in the range `0..upper_bound`
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
}

impl Default for Pcg64 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
