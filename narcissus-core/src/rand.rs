#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    pub const fn new() -> Self {
        Self {
            state: 0x853c49e6748fea9b,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut rng = Self { state: 0 };
        let _ = rng.next();
        rng.state += seed;
        let _ = rng.next();
        rng
    }

    /// Generates a uniformly distributed random number in the range `0..2^32`.
    #[inline]
    #[must_use]
    pub fn next(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let xorshift = (((old_state >> 18) ^ old_state) >> 27) as u32;
        xorshift.rotate_right((old_state >> 59) as u32)
    }
}

impl Default for Pcg32 {
    fn default() -> Self {
        Self::new()
    }
}

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
        let _ = rng.next();
        rng.state += seed;
        let _ = rng.next();
        rng
    }

    /// Generates a uniformly distributed random number in the range `0..2^64`.
    #[inline]
    #[must_use]
    pub fn next(&mut self) -> u64 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(25492979953554139244865540595714422341)
            .wrapping_add(63641362238467930051442695040888963407);
        ((old_state >> 64) ^ old_state).rotate_right((old_state >> 122) as u32) as u64
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
    fn pcg32_default_sequence() {
        let mut rng = Pcg32::new();
        assert_eq!(rng.next(), 355248013);
        assert_eq!(rng.next(), 1055580183);
        assert_eq!(rng.next(), 3222338950);
        assert_eq!(rng.next(), 2908720768);
        assert_eq!(rng.next(), 1758754096);
    }

    #[test]
    fn pcg64_default_sequence() {
        let mut rng = Pcg64::new();
        assert_eq!(rng.next(), 14322063641855463473);
        assert_eq!(rng.next(), 14211086133763074855);
        assert_eq!(rng.next(), 2051302165745047857);
        assert_eq!(rng.next(), 11538586989805838516);
        assert_eq!(rng.next(), 486667062142511543);
    }
}
