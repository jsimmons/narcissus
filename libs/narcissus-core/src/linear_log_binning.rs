#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bin<
    // The log2 of the size of the linear bin.
    const LINEAR_LOG2: u32,
    // The log2 of the number of sub-bins in each bin.
    const SUB_BINS_LOG2: u32,
> {
    pub index: u32,
}

impl<const LINEAR_LOG2: u32, const SUB_BINS_LOG2: u32> Bin<LINEAR_LOG2, SUB_BINS_LOG2> {
    pub const SUB_BIN_COUNT: u32 = 1 << SUB_BINS_LOG2;

    /// Create a bin from the given bin and sub-bin.
    pub fn new(bin: u32, sub_bin: u32) -> Self {
        debug_assert!(sub_bin < Self::SUB_BIN_COUNT);
        Self {
            index: (bin * Self::SUB_BIN_COUNT + sub_bin),
        }
    }

    /// Takes a size and returns the first bin whose entire range is large enough
    /// to contain it. That is, it rounds up.
    ///
    /// # Example
    ///
    /// ```
    /// // The log2 of the size of the 'linear' bin.
    /// pub const LINEAR_LOG2: u32 = 7; // 2^7 = 128
    /// // The log2 of the number of sub-bins in each bin.
    /// pub const SUB_BINS_LOG2: u32 = 5; // 2^5 = 32
    /// type Bin = narcissus_core::linear_log_binning::Bin<LINEAR_LOG2, SUB_BINS_LOG2>;
    /// assert_eq!(Bin::from_size_round_up(130), (132, Bin::new(1, 1)));
    /// ```
    #[inline(always)]
    pub fn from_size_round_up(size: u32) -> (u32, Self) {
        debug_assert!(size <= i32::MAX as u32);

        let num_bits = (size | 1 << LINEAR_LOG2).ilog2();
        let shift = num_bits - SUB_BINS_LOG2;
        let mask = (1 << shift) - 1;
        let rounded = size.wrapping_add(mask);
        let sub_index = rounded >> shift;
        let range = num_bits - LINEAR_LOG2;
        let index = (range << SUB_BINS_LOG2) + sub_index;
        let rounded_size = rounded & !mask;

        (rounded_size, Bin { index })
    }

    /// Takes a size and returns the bin whose range contains the given size. That
    /// is, it rounds down.
    ///
    /// # Example
    ///
    /// ```
    /// // The log2 of the size of the 'linear' bin.
    /// pub const LINEAR_LOG2: u32 = 7; // 2^7 = 128
    /// // The log2 of the number of sub-bins in each bin.
    /// pub const SUB_BINS_LOG2: u32 = 5; // 2^5 = 32
    /// type Bin = narcissus_core::linear_log_binning::Bin<LINEAR_LOG2, SUB_BINS_LOG2>;
    /// assert_eq!(Bin::from_size_round_down(130), (128, Bin::new(1, 0)));
    /// ```
    #[inline(always)]
    pub fn from_size_round_down(size: u32) -> (u32, Self) {
        debug_assert!(size <= i32::MAX as u32);

        let num_bits = (size | 1 << LINEAR_LOG2).ilog2();
        let shift = num_bits - SUB_BINS_LOG2;
        let sub_index = size >> shift;
        let range = num_bits - LINEAR_LOG2;

        let rounded_size = sub_index << shift;
        let index = (range << SUB_BINS_LOG2) + sub_index;

        (rounded_size, Bin { index })
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index as usize
    }

    #[inline(always)]
    pub fn bin(&self) -> u32 {
        self.index >> SUB_BINS_LOG2
    }

    #[inline(always)]
    pub fn sub_bin(&self) -> u32 {
        self.index & ((1 << SUB_BINS_LOG2) - 1) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const LINEAR_REGION_LOG2: u32 = 7;
    pub const SUB_BIN_COUNT_LOG2: u32 = 5;
    pub const SUB_BIN_COUNT: u32 = 1 << SUB_BIN_COUNT_LOG2;

    fn make_bin(
        rounded_size: u32,
        bin: u32,
        sub_bin: u32,
    ) -> (u32, Bin<LINEAR_REGION_LOG2, SUB_BIN_COUNT_LOG2>) {
        (
            rounded_size,
            Bin {
                index: bin * SUB_BIN_COUNT + sub_bin,
            },
        )
    }

    #[test]
    fn bin_from_size_round_up() {
        // Cases up to end of linear region.
        assert_eq!(Bin::from_size_round_up(0), make_bin(0, 0, 0));

        assert_eq!(Bin::from_size_round_up(1), make_bin(4, 0, 1));
        assert_eq!(Bin::from_size_round_up(2), make_bin(4, 0, 1));
        assert_eq!(Bin::from_size_round_up(3), make_bin(4, 0, 1));
        assert_eq!(Bin::from_size_round_up(4), make_bin(4, 0, 1));

        assert_eq!(Bin::from_size_round_up(5), make_bin(8, 0, 2));
        assert_eq!(Bin::from_size_round_up(6), make_bin(8, 0, 2));
        assert_eq!(Bin::from_size_round_up(7), make_bin(8, 0, 2));
        assert_eq!(Bin::from_size_round_up(8), make_bin(8, 0, 2));

        assert_eq!(Bin::from_size_round_up(121), make_bin(124, 0, 31));
        assert_eq!(Bin::from_size_round_up(122), make_bin(124, 0, 31));
        assert_eq!(Bin::from_size_round_up(123), make_bin(124, 0, 31));
        assert_eq!(Bin::from_size_round_up(124), make_bin(124, 0, 31));

        assert_eq!(Bin::from_size_round_up(125), make_bin(128, 1, 0));
        assert_eq!(Bin::from_size_round_up(126), make_bin(128, 1, 0));
        assert_eq!(Bin::from_size_round_up(127), make_bin(128, 1, 0));
        assert_eq!(Bin::from_size_round_up(128), make_bin(128, 1, 0));

        // Check all bin thresholds.
        for i in 0..32 - LINEAR_REGION_LOG2 - 1 {
            let bin = i + 1;
            let base = 1 << i + LINEAR_REGION_LOG2;
            let step = base >> SUB_BIN_COUNT_LOG2;
            for sub_bin in 0..SUB_BIN_COUNT as u32 {
                let size = base + sub_bin * step;
                assert_eq!(Bin::from_size_round_up(size), make_bin(size, bin, sub_bin));
                assert_eq!(make_bin(size, bin, sub_bin), (size, Bin::new(bin, sub_bin)));

                let next_size = base + (sub_bin + 1) * step;
                let next_bin = bin + (sub_bin == SUB_BIN_COUNT as u32 - 1) as u32;
                let next_sub_bin = (sub_bin + 1) % SUB_BIN_COUNT as u32;
                assert_eq!(
                    Bin::from_size_round_up(size + 1),
                    make_bin(next_size, next_bin, next_sub_bin)
                );
            }
        }
    }

    #[test]
    fn bin_from_size_round_down() {
        // Cases up to end of linear region.
        assert_eq!(Bin::from_size_round_down(0), make_bin(0, 0, 0));
        assert_eq!(Bin::from_size_round_down(1), make_bin(0, 0, 0));
        assert_eq!(Bin::from_size_round_down(2), make_bin(0, 0, 0));
        assert_eq!(Bin::from_size_round_down(3), make_bin(0, 0, 0));

        assert_eq!(Bin::from_size_round_down(4), make_bin(4, 0, 1));
        assert_eq!(Bin::from_size_round_down(5), make_bin(4, 0, 1));
        assert_eq!(Bin::from_size_round_down(6), make_bin(4, 0, 1));
        assert_eq!(Bin::from_size_round_down(7), make_bin(4, 0, 1));

        assert_eq!(Bin::from_size_round_down(124), make_bin(124, 0, 31));
        assert_eq!(Bin::from_size_round_down(125), make_bin(124, 0, 31));
        assert_eq!(Bin::from_size_round_down(126), make_bin(124, 0, 31));
        assert_eq!(Bin::from_size_round_down(127), make_bin(124, 0, 31));

        assert_eq!(Bin::from_size_round_down(128), make_bin(128, 1, 0));
        assert_eq!(Bin::from_size_round_down(129), make_bin(128, 1, 0));
        assert_eq!(Bin::from_size_round_down(130), make_bin(128, 1, 0));
        assert_eq!(Bin::from_size_round_down(131), make_bin(128, 1, 0));
    }
}
