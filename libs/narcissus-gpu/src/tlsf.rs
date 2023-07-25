//! Two Level Seggregated Fit Allocator
//! ===
//!
//! [Tlsf][tlsf] is a constant time, low fragmentation good-fit allocator based
//! on seggregated free-lists with a two-level bitmap acceleration structure.
//!
//! Memory is allocated by the underlying allocator into super-blocks,
//! representing large chunks of contiguous memory. The allocation routines
//! then work on blocks, which subdivide those regions.
//!
//! In order to quickly find a large-enough block, free blocks are stored in a
//! set of seggregated free-lists by their size. The requirements for a binning
//! strategy are as follows;
//!
//! 1) Must have a bounded number of bins.
//!
//! 2) Must be fast to find the bin for a given size.
//!
//! 3) Bin sizes must closely match allocation sizes to minimise fragmentation.
//!
//! For these purposes we use a [linear-log][linearlog] strategy for binning. An
//! initial 'linear' bin is divided into N sub-bins, then power-of-two sized
//! bins follow, also divided into N sub-bins. With some simple bit arithmetic
//! we can calculate the bucket for a given size.
//!
//! For example, if the initial linear region was 16, and the number of sub-bins
//! was 4, we would end up with a layout something like the following.
//!
//! ```text
//!                     1..=4       5..=8        9..=12      13..=16
//!                +------------+------------+------------+------------+
//! Linear Region  |    0x01    |    0x00    |    0x00    |    0x00    |
//!                +------------+------------+------------+------------+
//!
//!                   17..=20      21..=24      25..=28      29..=32
//!                +------------+------------+------------+------------+
//! 2^4            |    0x00    |    0x00    |    0x00    |    0x00    |
//!                +------------+------------+------------+------------+
//!
//!                   31..=40      41..=48      49..=56      57..=64
//!                +------------+------------+------------+------------+
//! 2^5            |    0x00    |    0x00    |    0x00    |    0x00    |
//!                +------------+------------+------------+------------+
//!
//!                   65..=80      81..=96      97..=112    113..=128
//!                +------------+------------+------------+------------+
//! 2^6            |    0x01    |    0x00    |    0x04    |    0x00    |
//!                +------------+------------+------------+------------+
//!
//! ```
//!
//! In order to avoid linearly scanning the free-lists to find suitable empty
//! blocks, we maintain a two-level bitmap acceleration structure. The first
//! level has a bit set for each non-empty bin, then the second level likewise
//! has a bit set for each non-empty sub-bin. From there it's possible to scan
//! with bit arithmetic to find the first suitable non-empty block without
//! traversing the entire free-lists structure.
//!
//! ```text
//!
//!               +---+---+---+---+
//! Level 0:      | 1 | 0 | 0 | 1 |                                          0x9
//!               +-+-+-+-+-+-+-+-+
//!                 |   |   |   |
//!                 |   |   |   |
//!                 |   |   |   |
//!                 |   |   |   |      +---+---+---+---+
//! Level 1:        |   |   |   +----->| 0 | 0 | 0 | 1 |   Linear Region     0x1
//!                 |   |   |          +---+---+---+---+
//!                 |   |   |
//!                 |   |   |          +---+---+---+---+
//!                 |   |   +--------->| 0 | 0 | 0 | 0 |   2^4               0x0
//!                 |   |              +---+---+---+---+
//!                 |   |
//!                 |   |              +---+---+---+---+
//!                 |   +------------->| 0 | 0 | 0 | 0 |   2^5               0x0
//!                 |                  +---+---+---+---+
//!                 |
//!                 |                  +---+---+---+---+
//!                 +----------------->| 0 | 1 | 0 | 0 |   2^6               0x4
//!                                    +---+---+---+---+
//!
//! ```
//!
//! [tlsf]: http://www.gii.upv.es/tlsf/files/spe_2008.pdf
//! [linearlog]: https://pvk.ca/Blog/2015/06/27/linear-log-bucketing-fast-versatile-simple/

use std::{
    num::NonZeroU32,
    ops::{Index, IndexMut},
};

use narcissus_core::{linear_log_binning, static_assert, Widen};

// The log2 of the size of the 'linear' bin.
pub const LINEAR_LOG2: u32 = 9; // 2^9 = 512

// The log2 of the number of sub-bins in each bin.
pub const SUB_BINS_LOG2: u32 = 5; // 2^5 = 32

static_assert!(LINEAR_LOG2 >= SUB_BINS_LOG2);

type Bin = linear_log_binning::Bin<LINEAR_LOG2, SUB_BINS_LOG2>;

pub const BIN_COUNT: usize = (u32::BITS - LINEAR_LOG2) as usize;
pub const SUB_BIN_COUNT: usize = 1 << SUB_BINS_LOG2;

static_assert!(SUB_BIN_COUNT <= u32::BITS as usize);
static_assert!(BIN_COUNT <= u32::BITS as usize);

pub const MIN_ALIGNMENT: u32 = 1 << (LINEAR_LOG2 - SUB_BINS_LOG2);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct SuperBlockIndex(u32);

pub struct SuperBlock<T>
where
    T: Copy + Default,
{
    _first_block_index: BlockIndex,
    pub user_data: T,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct BlockIndex(NonZeroU32);

const INVALID_BLOCK_INDEX: BlockIndex = BlockIndex(match NonZeroU32::new(0xffff_ffff) {
    Some(x) => x,
    None => panic!(),
});

struct BlockLink {
    prev: BlockIndex,
    next: BlockIndex,
}

impl BlockLink {
    /// Create a new unlinked BlockLink for the given `block_index`.
    const fn new(block_index: BlockIndex) -> Self {
        Self {
            prev: block_index,
            next: block_index,
        }
    }

    /// Returns true if the given link is not inserted into any list.
    fn is_unlinked(&self) -> bool {
        self.prev == self.next
    }
}

/// Insert the node at index `$insert` before the node at index `$x` for the
/// list given by `$storage` and `$link_name`.
macro_rules! list_insert_before {
    ($storage:expr, $link_name:ident, $x:expr, $insert:expr) => {
        $storage[$insert].$link_name.prev = $storage[$x].$link_name.prev;
        $storage[$insert].$link_name.next = $x;
        let prev_index = $storage[$insert].$link_name.prev;
        $storage[prev_index].$link_name.next = $insert;
        let next_index = $storage[$insert].$link_name.next;
        $storage[next_index].$link_name.prev = $insert;
    };
}

/// Insert the node at index `$insert` after the node at index `$x` for the
/// list given by `$storage` and `$link_name`.
macro_rules! list_insert_after {
    ($storage:expr, $link_name:ident, $x:expr, $insert:expr) => {
        $storage[$insert].$link_name.prev = $x;
        $storage[$insert].$link_name.next = $storage[$x].$link_name.next;
        let prev_index = $storage[$insert].$link_name.prev;
        $storage[prev_index].$link_name.next = $insert;
        let next_index = $storage[$insert].$link_name.next;
        $storage[next_index].$link_name.prev = $insert;
    };
}

/// Unlink the node`$x` for the list given by `$storage` and `$link_name`.
macro_rules! list_unlink {
    ($storage:expr, $link_name:ident, $x:expr) => {
        let prev_index = $storage[$x].$link_name.prev;
        $storage[prev_index].$link_name.next = $storage[$x].$link_name.next;
        let next_index = $storage[$x].$link_name.next;
        $storage[next_index].$link_name.prev = $storage[$x].$link_name.prev;
        $storage[$x].$link_name.prev = $x;
        $storage[$x].$link_name.next = $x;
    };
}

struct Block {
    size: u32,
    offset: u32,
    generation: u32,
    super_block_index: SuperBlockIndex,

    free_link: BlockLink,
    phys_link: BlockLink,
}

const DUMMY_BLOCK: Block = Block {
    generation: 0xffff_ffff,
    size: 0xffff_ffff,
    offset: 0xffff_ffff,
    free_link: BlockLink::new(INVALID_BLOCK_INDEX),
    phys_link: BlockLink::new(INVALID_BLOCK_INDEX),
    super_block_index: SuperBlockIndex(0xffff_ffff),
};

impl Block {
    fn is_used(&self) -> bool {
        self.generation & 1 == 1
    }

    fn is_free(&self) -> bool {
        self.generation & 1 == 0
    }
}

impl Index<BlockIndex> for Vec<Block> {
    type Output = Block;

    #[inline(always)]
    fn index(&self, index: BlockIndex) -> &Self::Output {
        &self[index.0.get().widen()]
    }
}

impl IndexMut<BlockIndex> for Vec<Block> {
    #[inline(always)]
    fn index_mut(&mut self, index: BlockIndex) -> &mut Self::Output {
        &mut self[index.0.get().widen()]
    }
}

impl<T> Index<SuperBlockIndex> for Vec<SuperBlock<T>>
where
    T: Copy + Default,
{
    type Output = SuperBlock<T>;

    #[inline(always)]
    fn index(&self, index: SuperBlockIndex) -> &Self::Output {
        &self[index.0.widen()]
    }
}

impl<T> IndexMut<SuperBlockIndex> for Vec<SuperBlock<T>>
where
    T: Copy + Default,
{
    #[inline(always)]
    fn index_mut(&mut self, index: SuperBlockIndex) -> &mut Self::Output {
        &mut self[index.0.widen()]
    }
}

#[derive(Clone)]
pub struct Allocation<T> {
    block_index: BlockIndex,
    generation: u32,
    offset: u64,
    user_data: T,
}

impl<T> Allocation<T> {
    pub fn user_data(&self) -> &T {
        &self.user_data
    }

    /// Returns the offset into the super-block where this allocation starts.
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

pub struct Tlsf<T>
where
    T: Copy + Default,
{
    bitmap_0: u32,
    bitmap_1: [u32; BIN_COUNT],
    empty_block_heads: [Option<BlockIndex>; SUB_BIN_COUNT * BIN_COUNT],

    free_block_head: Option<BlockIndex>,
    blocks: Vec<Block>,

    super_blocks: Vec<SuperBlock<T>>,
}

impl<T> Default for Tlsf<T>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Tlsf<T>
where
    T: Copy + Default,
{
    pub fn new() -> Self {
        Self {
            bitmap_0: 0,
            bitmap_1: [0; BIN_COUNT],
            empty_block_heads: [None; SUB_BIN_COUNT * BIN_COUNT],
            free_block_head: None,
            blocks: vec![DUMMY_BLOCK],
            super_blocks: vec![],
        }
    }

    /// Returns a slice containing all the super_blocks added to the allocator.
    /// Only the `user_data` field is accessible.
    pub fn super_blocks(&self) -> &[SuperBlock<T>] {
        &self.super_blocks
    }

    /// Clear the allocator state.
    ///
    /// Make sure to clean up any super blocks before calling this.
    pub fn clear(&mut self) {
        self.bitmap_0 = 0;
        self.bitmap_1.fill(0);
        self.empty_block_heads.fill(None);
        self.free_block_head = None;
        self.blocks.clear();
        self.blocks.push(DUMMY_BLOCK);
        self.super_blocks.clear()
    }

    /// Search the acceleration structure for a non-empty list suitable for an
    /// allocation of the given size.
    ///
    /// Returns the rounded size and bin index if a non-empty list is found, or
    /// None.
    fn search_non_empty_bin(&self, size: u32) -> Option<(u32, Bin)> {
        // We need to find the bin which contains only empty-blocks large enough for the
        // given size because we unconditionally use the first empty block found. So
        // this must round up.
        let (rounded_size, starting_bin) = Bin::from_size_round_up(size);

        let mut bin = starting_bin.bin();
        let sub_bin = starting_bin.sub_bin();

        // First we scan the second-level bitmap from sub_bin, masking out the earlier
        // sub-bins so we don't end up returning a bin that's too small for the
        // allocation.
        let mut second_level = self.bitmap_1[bin.widen()] & (!0 << sub_bin);

        // If that search failed, then we must scan the first-level bitmap from the next
        // bin forward. If we find anything here it cannot possibly be smaller than the
        // requested allocation.
        if second_level == 0 {
            let first_level = self.bitmap_0 & (!0 << (bin + 1));

            // If that search also failed, there's no suitable blocks.
            if first_level == 0 {
                return None;
            }

            // Recalculate the bin from the first level bitmap.
            bin = first_level.trailing_zeros();
            second_level = self.bitmap_1[bin.widen()];
        }

        // Find the sub-bin from the second level bitmap.
        let sub_bin = second_level.trailing_zeros();
        Some((rounded_size, Bin::new(bin, sub_bin)))
    }

    /// Marks a given bin as containing empty blocks in the bitmap acceleration
    /// structure.
    fn set_metadata_bit(&mut self, bin: Bin) {
        let sub_bin = bin.sub_bin();
        let bin = bin.bin().widen();
        self.bitmap_0 |= 1 << bin;
        self.bitmap_1[bin] |= 1 << sub_bin;
    }

    /// Marks a given bin as containing no empty blocks in the bitmap acceleration
    /// structure.
    fn clear_metadata_bit(&mut self, bin: Bin) {
        let sub_bin = bin.sub_bin();
        let bin = bin.bin().widen();
        self.bitmap_1[bin] &= !(1 << sub_bin);
        if self.bitmap_1[bin] == 0 {
            self.bitmap_0 &= !(1 << bin);
        }
    }

    /// Inserts a block into the empty blocks lists.
    #[inline(always)]
    fn insert_block(&mut self, block_index: BlockIndex) {
        debug_assert!(self.blocks[block_index].is_free());
        debug_assert!(self.blocks[block_index].free_link.is_unlinked());

        let (_, bin) = Bin::from_size_round_down(self.blocks[block_index].size);
        let bin_index = bin.index();

        if let Some(empty_block_index) = self.empty_block_heads[bin_index] {
            list_insert_before!(self.blocks, free_link, empty_block_index, block_index);
        } else {
            self.set_metadata_bit(bin);
        }

        self.empty_block_heads[bin_index] = Some(block_index);
    }

    /// Removes a block from the empty blocks lists.
    #[inline(always)]
    fn extract_block(&mut self, block_index: BlockIndex) {
        debug_assert!(self.blocks[block_index].is_free());

        let (_, bin) = Bin::from_size_round_down(self.blocks[block_index].size);

        let bin_index = bin.index();

        debug_assert!(self.empty_block_heads[bin_index].is_some());

        if self.empty_block_heads[bin_index] == Some(block_index) {
            let next_index = self.blocks[block_index].free_link.next;
            if next_index != block_index {
                self.empty_block_heads[bin_index] = Some(next_index);
            } else {
                self.empty_block_heads[bin_index] = None;
                self.clear_metadata_bit(bin);
            }
        }

        list_unlink!(self.blocks, free_link, block_index);
    }

    /// Returns true if we should merge `from_block_index` into `into_block_index`.
    fn can_merge_block_left(
        &self,
        into_block_index: BlockIndex,
        from_block_index: BlockIndex,
    ) -> bool {
        // Cannot merge into ourselves.
        if into_block_index == from_block_index {
            return false;
        }

        // Cannot merge the first block in a physical range into the last block.
        // This check is necessary because the linked lists are cyclic.
        if self.blocks[from_block_index].offset == 0 {
            return false;
        }

        // Cannot merge blocks that are in-use.
        if self.blocks[into_block_index].is_used() || self.blocks[from_block_index].is_used() {
            return false;
        }

        true
    }

    /// Requests a new block, and returns its `BlockIndex`.
    #[inline(always)]
    fn request_block(
        &mut self,
        offset: u32,
        size: u32,
        super_block_index: SuperBlockIndex,
    ) -> BlockIndex {
        #[cold]
        fn create_block(blocks: &mut Vec<Block>) -> BlockIndex {
            assert!(blocks.len() < i32::MAX as usize);
            let block_index = BlockIndex(NonZeroU32::new(blocks.len() as u32).unwrap());
            blocks.push(Block {
                generation: 0,
                size: 0xffff_ffff,
                offset: 0xffff_ffff,
                free_link: BlockLink::new(block_index),
                phys_link: BlockLink::new(block_index),
                super_block_index: SuperBlockIndex(0xffff_ffff),
            });
            block_index
        }

        let block_index = if let Some(free_block_index) = self.free_block_head {
            let next_index = self.blocks[free_block_index].free_link.next;
            self.free_block_head = if next_index != free_block_index {
                Some(next_index)
            } else {
                None
            };
            list_unlink!(self.blocks, free_link, free_block_index);
            free_block_index
        } else {
            create_block(&mut self.blocks)
        };

        let block = &mut self.blocks[block_index];

        debug_assert!(block.is_free());
        debug_assert!(block.size == 0xffff_ffff);
        debug_assert!(block.offset == 0xffff_ffff);
        debug_assert!(block.super_block_index == SuperBlockIndex(0xffff_ffff));

        block.offset = offset;
        block.size = size;
        block.super_block_index = super_block_index;

        block_index
    }

    /// Recycles the block indicated by `block_index` for re-use.
    fn recycle_block(&mut self, block_index: BlockIndex) {
        let block = &mut self.blocks[block_index];
        debug_assert!(block.free_link.is_unlinked());
        debug_assert!(block.phys_link.is_unlinked());

        block.size = 0xffff_ffff;
        block.offset = 0xffff_ffff;
        block.super_block_index = SuperBlockIndex(0xffff_ffff);

        if let Some(free_block_index) = self.free_block_head {
            list_insert_before!(self.blocks, free_link, free_block_index, block_index);
        }

        self.free_block_head = Some(block_index);
    }

    pub fn insert_super_block(&mut self, size: u64, user_data: T) {
        assert!(size != 0 && size < i32::MAX as u64);
        assert!(self.super_blocks.len() < i32::MAX as usize);

        // Ranges checked in asserts above.
        let size = size as u32;
        let len = self.super_blocks.len() as u32;

        let super_block_index = SuperBlockIndex(len);
        let block_index = self.request_block(0, size, super_block_index);

        self.super_blocks.push(SuperBlock {
            // The block at offset 0 in a SuperBlock will never be merged away, so the index
            // is stable and we can store it in the SuperBlock itself.
            _first_block_index: block_index,
            user_data,
        });

        self.insert_block(block_index);
    }

    pub fn alloc(&mut self, size: u64, align: u64) -> Option<Allocation<T>> {
        assert!(
            size != 0
                && align != 0
                && align < i32::MAX as u64
                && size < (i32::MAX as u64 - align)
                && align.is_power_of_two()
        );
        let size = size.max(MIN_ALIGNMENT as u64);
        let size = if align > MIN_ALIGNMENT as u64 {
            size - 1 + align
        } else {
            size
        } as u32;

        let Some((rounded_size, bin)) = self.search_non_empty_bin(size) else {
            return None;
        };

        let block_index = self.empty_block_heads[bin.index()].unwrap();

        debug_assert!(
            self.blocks[block_index].is_free() && self.blocks[block_index].size >= rounded_size
        );

        self.extract_block(block_index);

        // It's important to use the rounded size here, not the requested size. This
        // avoids a failure case where freeing an allocation of a given size, fails
        // to leave the allocator in a state where that block can be re-used for
        // another allocation of the same size as the returned block can be placed into
        // a smaller bin.
        //
        // We're trading arbitrary wasted blocks in this workload, for a small bounded
        // amount of fragmentation.
        //
        // Tested in `tests::split_policy_avoids_memory_waste`
        let remainder = self.blocks[block_index].size - rounded_size;
        let super_block_index = self.blocks[block_index].super_block_index;

        // Should we should split the block?
        if remainder >= MIN_ALIGNMENT {
            self.blocks[block_index].size -= remainder;
            let offset = self.blocks[block_index].offset + rounded_size;
            let new_block_index = self.request_block(offset, remainder, super_block_index);
            list_insert_after!(self.blocks, phys_link, block_index, new_block_index);
            self.insert_block(new_block_index);
        }

        let generation = self.blocks[block_index].generation.wrapping_add(1);
        self.blocks[block_index].generation = generation;

        let user_data = self.super_blocks[super_block_index].user_data;
        // The mask is a no-op if the alignment is already met, do it unconditionally.
        let offset = (self.blocks[block_index].offset as u64 + align - 1) & !(align - 1);

        debug_assert_eq!(offset & (align - 1), 0);

        Some(Allocation {
            block_index,
            generation,
            offset,
            user_data,
        })
    }

    pub fn free(&mut self, allocation: Allocation<T>) {
        let mut block_index = allocation.block_index;
        let generation = self.blocks[block_index].generation;
        assert_eq!(generation, allocation.generation, "double-free");
        self.blocks[block_index].generation = generation.wrapping_add(1);

        // Merge next block into the current block.
        {
            let into_block_index = block_index;
            let from_block_index = self.blocks[block_index].phys_link.next;
            if self.can_merge_block_left(into_block_index, from_block_index) {
                let from_size = self.blocks[from_block_index].size;
                self.extract_block(from_block_index);
                list_unlink!(self.blocks, phys_link, from_block_index);
                self.recycle_block(from_block_index);
                self.blocks[into_block_index].size += from_size;
            }
        }

        // Merge current block into the prev block.
        {
            let into_block_index = self.blocks[block_index].phys_link.prev;
            let from_block_index = block_index;
            if self.can_merge_block_left(into_block_index, from_block_index) {
                let from_size = self.blocks[from_block_index].size;
                self.extract_block(into_block_index);
                list_unlink!(self.blocks, phys_link, from_block_index);
                self.recycle_block(from_block_index);
                self.blocks[into_block_index].size += from_size;
                block_index = into_block_index;
            }
        }

        // Insert the merged free block.
        self.insert_block(block_index);
    }

    #[cfg(debug_assertions)]
    pub fn debug_bitmap_svg(&self, w: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        use narcissus_core::svg::{self, svg_begin, svg_end};

        struct Bytes {
            bytes: u32,
        }

        impl Bytes {
            fn new(bytes: u32) -> Self {
                Self { bytes }
            }
        }

        impl std::fmt::Display for Bytes {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.bytes < 1024 {
                    write!(f, "{}b", self.bytes)
                } else if self.bytes < 1024 * 1024 {
                    write!(f, "{:.2}KiB", self.bytes as f32 / (1024.0))
                } else {
                    write!(f, "{:.2}MiB", self.bytes as f32 / (1024.0 * 1024.0))
                }
            }
        }

        write!(w, "{}", svg_begin(615.0, 375.0))?;

        const BOX_SIZE: f32 = 15.0;
        const PAD: f32 = 30.0;

        let stroke = svg::stroke(svg::black(), 2.0, 1.0);
        let fg = svg::style(svg::fill(svg::rgb(0xdf, 0x73, 0x1a), 1.0), stroke);
        let bg = svg::style(svg::fill(svg::rgb(0xfe, 0xfe, 0xfe), 0.0), stroke);

        let mut y = 28.0;
        let mut x = 0.0;

        for i in 0..BIN_COUNT {
            let bin = Bin::new(i as u32, 0);
            write!(
                w,
                "{}",
                svg::text(x, y, 14.0, fg, &Bytes::new(bin.lower_bound()))
            )?;
            y += BOX_SIZE;
        }

        y = PAD;
        x = 100.0;

        for i in 0..BIN_COUNT {
            let empty = self.bitmap_0 & 1 << i == 0;
            write!(
                w,
                "{}",
                svg::rect(x, y, BOX_SIZE, BOX_SIZE).style(if empty { bg } else { fg })
            )?;
            y += BOX_SIZE;
        }

        y = PAD;
        x = 100.0 + PAD * 2.0;

        for (bin, bitmap) in self.bitmap_1.iter().enumerate() {
            for sub_bin in 0..SUB_BIN_COUNT {
                let bin = Bin::new(bin as u32, sub_bin as u32);
                let lower_bound = Bytes::new(bin.lower_bound());
                let upper_bound = Bytes::new(bin.upper_bound());
                let range = format!("{lower_bound}-{upper_bound}");

                let empty = bitmap & 1 << sub_bin == 0;

                write!(
                    w,
                    "{}",
                    svg::rect(x, y, BOX_SIZE, BOX_SIZE)
                        .style(if empty { bg } else { fg })
                        .title(&range)
                )?;
                x += BOX_SIZE;
            }
            x = 100.0 + PAD * 2.0;
            y += BOX_SIZE;
        }

        write!(w, "{}", svg_end())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use narcissus_core::rand::Pcg64;

    use super::*;

    #[test]
    fn split_and_merge() {
        let mut tlsf = Tlsf::new();

        tlsf.insert_super_block(1024, ());

        let alloc0 = tlsf.alloc(512, 1).unwrap();
        let alloc1 = tlsf.alloc(512, 1).unwrap();
        assert!(tlsf.alloc(512, 1).is_none());

        // Freeing should merge the blocks.

        tlsf.free(alloc0);
        tlsf.free(alloc1);

        // and allow us to allocate the full size again.
        let alloc2 = tlsf.alloc(1024, 1).unwrap();
        assert!(tlsf.alloc(512, 1).is_none());
        tlsf.free(alloc2);

        {
            let mut allocations = (0..64)
                .map(|_| tlsf.alloc(16, 1).unwrap())
                .collect::<Vec<_>>();

            assert!(tlsf.alloc(16, 1).is_none());

            for allocation in allocations.drain(..).rev() {
                tlsf.free(allocation);
            }
        }

        // and allow us to allocate the full size again.
        let alloc2 = tlsf.alloc(1024, 1).unwrap();
        assert!(tlsf.alloc(512, 1).is_none());
        tlsf.free(alloc2);

        {
            let mut allocations = (0..64)
                .map(|_| tlsf.alloc(16, 1).unwrap())
                .collect::<Vec<_>>();

            assert!(tlsf.alloc(16, 1).is_none());

            for allocation in allocations.drain(..) {
                tlsf.free(allocation);
            }
        }

        // and allow us to allocate the full size again.
        let alloc2 = tlsf.alloc(1024, 1).unwrap();
        assert!(tlsf.alloc(512, 1).is_none());
        tlsf.free(alloc2);
    }

    #[test]
    fn multiple_super_blocks() {
        let mut tlsf = Tlsf::new();

        const NUM_SUPER_BLOCKS: u64 = 16;
        const SUPER_BLOCK_SIZE: u64 = 10 * 1024;

        const TOTAL_SIZE: u64 = NUM_SUPER_BLOCKS * SUPER_BLOCK_SIZE;
        const ALLOCATION_SIZE: u64 = 16;

        for _ in 0..NUM_SUPER_BLOCKS {
            tlsf.insert_super_block(SUPER_BLOCK_SIZE, ());
        }

        let mut seed_rng = Pcg64::new();

        for _run in 0..4 {
            let seed = seed_rng.next_u64() as u128 | (seed_rng.next_u64() as u128) << 64;
            let mut rng = Pcg64::with_seed(seed);

            let mut allocations = (0..(TOTAL_SIZE / ALLOCATION_SIZE))
                .map(|_| tlsf.alloc(ALLOCATION_SIZE, 1).unwrap())
                .collect::<Vec<_>>();

            rng.shuffle(allocations.as_mut_slice());

            for allocation in allocations.drain(..) {
                tlsf.free(allocation);
            }
        }
    }

    #[test]
    fn split_policy_avoids_memory_waste() {
        let mut tlsf = Tlsf::new();
        tlsf.insert_super_block(1024, ());

        let large_size = 990;
        let small_size = 30;

        // Make a large allocation that splits the block.
        let large = tlsf.alloc(large_size, 1).unwrap();
        // Make a small allocation to inhibit merging upon free.
        tlsf.alloc(small_size, 1).unwrap();
        // Free the large block, if all goes well this will be added to a bin which is
        // large enough to service another allocation of the same size.
        tlsf.free(large);
        // Allocate another large block, if this fails we've "lost" memory.
        tlsf.alloc(large_size, 1).unwrap();
    }

    #[test]
    #[should_panic]
    fn double_free() {
        let mut tlsf = Tlsf::new();
        tlsf.insert_super_block(1024, ());
        let alloc = tlsf.alloc(512, 1).unwrap();
        tlsf.free(alloc.clone());
        tlsf.free(alloc);
    }
}
