use std::{marker::PhantomData, mem::size_of, ptr::NonNull};

use crate::{
    align_offset, mod_inverse_u32, static_assert, virtual_commit, virtual_free, virtual_reserve,
};

/// Each handle uses `GEN_BITS` bits of per-slot generation counter. Looking up a handle with the
/// correct index but an incorrect generation will yield `None`.
const GEN_BITS: u32 = 9;

/// Each handle uses `IDX_BITS` bits of index used to select a slot. This limits the maximum
/// capacity of the table to `2 ^ IDX_BITS - 1`.
const IDX_BITS: u32 = 23;

const MAX_IDX: usize = 1 << IDX_BITS as usize;
/// Reserve the last slot for the null handle.
const MAX_CAP: usize = MAX_IDX - 1;

const PAGE_SIZE: usize = 4096;

/// Keep at least `MIN_FREE_SLOTS` available at all times in order to ensure a minimum of
/// `MIN_FREE_SLOTS * 2 ^ (GEN_BITS - 1)` create-delete cycles are required before a duplicate handle is
/// generated.
const MIN_FREE_SLOTS: usize = 512;

static_assert!(GEN_BITS + IDX_BITS == 32);

const GEN_MASK: u32 = (1 << GEN_BITS) - 1;
const IDX_MASK: u32 = (1 << IDX_BITS) - 1;

const IDX_SHIFT: u32 = 0;
const GEN_SHIFT: u32 = IDX_SHIFT + IDX_BITS;

/// A handle representing an object stored in the associated pool.
///
/// Although the handle is mixed based on a per-pool random number, it's recommended to additionally create a newtype
/// wrapper around this type, to provide type safety preventing the handles from separate pools from becoming confused.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(u32);

impl Default for Handle {
    fn default() -> Self {
        Self::null()
    }
}

impl Handle {
    /// Create a handle from the given encode_multiplier, generation counter and slot index.
    ///
    /// # Panics
    ///
    /// Panics if the generation counter is even, as that would reference an empty slot.
    #[inline(always)]
    fn encode(encode_multiplier: u32, generation: u32, slot_index: SlotIndex) -> Self {
        assert!(generation & 1 == 1);

        let value = (generation & GEN_MASK) << GEN_SHIFT | (slot_index.0 & IDX_MASK) << IDX_SHIFT;
        // Invert bits so that the all bits set, the null handle, becomes zero.
        let value = !value;
        // Transform by the per-pool multiplier to mix bits such that handles from different pools are unlikely to collide.
        // Note this will return 0 for the null handle due to the inversion above.
        let value = value.wrapping_mul(encode_multiplier);
        Self(value)
    }

    /// Return a tuple containing the generation counter and slot index from an encoded handle and decode multiplier.
    ///
    /// # Panics
    ///
    /// Panics if the generation counter is even, as that would reference an empty slot.
    fn decode(self, decode_multiplier: u32) -> (u32, SlotIndex) {
        let value = self.0;
        // Undo the bit mix from the encode step by multiplying by the multiplicative inverse of the encode_multiplier.
        let value = value.wrapping_mul(decode_multiplier);
        // Invert bits so zero, the null handle, becomes all bits set.
        let value = !value;
        let generation = (value >> GEN_SHIFT) & GEN_MASK;
        let slot_index = SlotIndex((value >> IDX_SHIFT) & IDX_MASK);

        // An invalid generation counter here means either the handle itself has been corrupted, or that it's from
        // another pool.
        assert!(generation & 1 == 1, "invalid generation counter");

        (generation, slot_index)
    }

    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            f.debug_tuple("Handle").field(&"null").finish()
        } else {
            f.debug_struct("Handle").field("encoded", &self.0).finish()
        }
    }
}

/// A strongly typed index in the slots array.
#[derive(Clone, Copy, PartialEq, Eq)]
struct SlotIndex(u32);

/// A strongly typed index in the values array.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ValueIndex(u32);

impl ValueIndex {
    fn invalid() -> Self {
        Self(!0)
    }
}

/// Packed value storing the generation and value index for each slot in the indirection table.
///
/// The least-significant bit of the generation counter serves to indicate whether the slot is occupied. If it's 1,
/// the slot contains a valid entry. If it's 0, the slot is invalid.
struct Slot {
    value_index_and_gen: u32,
}

impl Slot {
    #[inline(always)]
    const fn new() -> Self {
        Self {
            // Clear the generation counter, but leave the index bits set.
            value_index_and_gen: IDX_MASK,
        }
    }

    /// Extract the current generation counter from this slot.
    #[inline(always)]
    fn generation(&self) -> u32 {
        (self.value_index_and_gen >> GEN_SHIFT) & GEN_MASK
    }

    /// Extract the current value index from this slot.
    #[inline(always)]
    fn value_index(&self) -> ValueIndex {
        ValueIndex((self.value_index_and_gen >> IDX_SHIFT) & IDX_MASK)
    }

    /// Updates the slot's value index without modifying the generation.
    #[inline(always)]
    fn update_value_index(&mut self, value_index: ValueIndex) {
        debug_assert!(value_index.0 & IDX_MASK == value_index.0);
        self.value_index_and_gen =
            self.generation() << GEN_SHIFT | (value_index.0 & IDX_MASK) << IDX_SHIFT;
    }

    /// Sets the slot's value index, incrementing the generation counter.
    #[inline(always)]
    fn set_value_index(&mut self, value_index: ValueIndex) {
        let new_generation = self.generation().wrapping_add(1);
        self.value_index_and_gen =
            (new_generation & GEN_MASK) << GEN_SHIFT | (value_index.0 & IDX_MASK) << IDX_SHIFT;
    }

    /// Clears the slot's value index, incrementing the generation counter.
    #[inline(always)]
    fn clear_value_index(&mut self) {
        // Since we're clearing we need to reset the generation to one referencing an empty slot. But we still want to
        // invalidate old handles.
        let new_generation = (self.generation() | 1).wrapping_add(1);
        self.value_index_and_gen = (new_generation & GEN_MASK) << GEN_SHIFT | IDX_MASK << IDX_SHIFT;
    }
}

/// FIFO free list of slot indices.
struct FreeSlots {
    head: usize,
    tail: usize,
    cap: usize,
    ptr: NonNull<SlotIndex>,
}

impl FreeSlots {
    fn new(ptr: NonNull<SlotIndex>) -> Self {
        Self {
            head: 0,
            tail: 0,
            cap: 0,
            ptr,
        }
    }

    #[inline]
    fn head(&self) -> usize {
        self.head & (self.cap - 1)
    }

    #[inline]
    fn tail(&self) -> usize {
        self.tail & (self.cap - 1)
    }

    #[inline]
    fn len(&self) -> usize {
        self.head.wrapping_sub(self.tail)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.len() == self.cap
    }

    fn push(&mut self, free_slot_index: SlotIndex) {
        if self.is_full() {
            self.grow();
        }

        let head = self.head();
        self.head = self.head.wrapping_add(1);
        unsafe { std::ptr::write(self.ptr.as_ptr().add(head), free_slot_index) }
    }

    fn pop(&mut self) -> Option<SlotIndex> {
        if self.is_empty() {
            return None;
        }
        let tail = self.tail();
        self.tail = self.tail.wrapping_add(1);
        Some(unsafe { std::ptr::read(self.ptr.as_ptr().add(tail)) })
    }

    #[cold]
    fn grow(&mut self) {
        // Free slots must always be a power of two so that the modular arithmetic for indexing
        // works out correctly.
        debug_assert!(self.cap == 0 || self.cap.is_power_of_two());
        assert!(self.cap <= MAX_IDX, "freelist overflow");

        let new_cap = if self.cap == 0 { 1024 } else { self.cap << 1 };
        unsafe {
            virtual_commit(
                self.ptr.as_ptr().add(self.cap) as _,
                (new_cap - self.cap) * size_of::<u32>(),
            )
        };

        // This is slightly wrong, but our freelist doesn't need correct ordering on resize and this
        // avoids moving the values around.
        if !self.is_empty() {
            debug_assert!(self.is_full());
            self.tail = 0;
            self.head = self.cap;
        }

        self.cap = new_cap;
    }
}

// Make sure the slots array always grows by a single page.
const SLOT_GROWTH_AMOUNT: usize = PAGE_SIZE / std::mem::size_of::<Slot>();

/// Indirection table mapping slot indices stored in handles to values in the values array.
///
/// Also contains the generation counter for each slot.
struct Slots {
    len: usize,
    ptr: NonNull<Slot>,
}

impl Slots {
    fn new(ptr: NonNull<Slot>) -> Self {
        Self { len: 0, ptr }
    }

    #[inline(always)]
    fn get(&self, slot_index: SlotIndex) -> Option<&Slot> {
        let index = slot_index.0 as usize;
        if index < self.len {
            Some(unsafe { self.ptr.as_ptr().add(index).as_ref().unwrap() })
        } else {
            None
        }
    }

    #[inline(always)]
    fn get_mut(&mut self, slot_index: SlotIndex) -> Option<&mut Slot> {
        let index = slot_index.0 as usize;
        if index < self.len {
            Some(unsafe { self.ptr.as_ptr().add(index).as_mut().unwrap() })
        } else {
            None
        }
    }

    /// Attempts to grow the slots array.
    ///
    /// Returns a tuple containing the old len and new len, or None if the array was already at capacity.
    #[cold]
    fn try_grow(&mut self) -> Option<(u32, u32)> {
        let len = self.len;
        let new_len = std::cmp::min(len + SLOT_GROWTH_AMOUNT, MAX_CAP);
        if new_len <= len {
            return None;
        }

        unsafe {
            virtual_commit(
                self.ptr.as_ptr().add(len) as _,
                (new_len - len) * size_of::<Slot>(),
            );
            for new_slot_index in len..new_len {
                std::ptr::write(self.ptr.as_ptr().add(new_slot_index), Slot::new());
            }
        }

        self.len = new_len;
        Some((len as u32, new_len as u32))
    }
}

/// A contiguous growable array of values as well as a reverse-lookup table for slot indices that map to those values.
struct Values<T> {
    cap: usize,
    len: usize,
    slots_ptr: NonNull<SlotIndex>,
    values_ptr: NonNull<T>,
    phantom: PhantomData<T>,
}

impl<T> Values<T> {
    fn new(slots_ptr: NonNull<SlotIndex>, values_ptr: NonNull<T>) -> Self {
        Self {
            cap: 0,
            len: 0,
            slots_ptr,
            values_ptr,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.values_ptr.as_ptr(), self.len) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.values_ptr.as_ptr(), self.len) }
    }

    /// Update the lookup table for the given `ValueIndex` with a new `SlotIndex`
    #[inline(always)]
    fn set_slot(&mut self, value_index: ValueIndex, slot_index: SlotIndex) {
        let value_index = value_index.0 as usize;
        assert!(value_index < self.len);
        unsafe {
            std::ptr::write(
                self.slots_ptr.as_ptr().add(value_index).as_mut().unwrap(),
                slot_index,
            )
        }
    }

    /// Retreive the `SlotIndex` corresponding to the given `ValueIndex` from the lookup table.
    #[inline(always)]
    fn get_slot(&mut self, value_index: ValueIndex) -> SlotIndex {
        let value_index = value_index.0 as usize;
        assert!(value_index < self.len);
        // SAFETY: SlotIndex is Copy so we don't invalidate the value being read.
        unsafe { std::ptr::read(self.slots_ptr.as_ptr().add(value_index).as_ref().unwrap()) }
    }

    /// Push a new value into the values storage. Returns the index of the added value.
    #[inline(always)]
    fn push(&mut self, value: T) -> ValueIndex {
        if self.len == self.cap {
            self.grow();
        }

        let new_value_index = self.len;
        self.len += 1;
        unsafe { std::ptr::write(self.values_ptr.as_ptr().add(new_value_index), value) };

        ValueIndex(new_value_index as u32)
    }

    /// Remove the element at the given `ValueIndex` and replace it with the last element. Fixup
    /// the lookup tables for the moved element.
    ///
    /// Returns the removed value.
    #[inline(always)]
    fn swap_remove(&mut self, value_index: ValueIndex, slots: &mut Slots) -> T {
        let last_value_index = ValueIndex((self.len - 1) as u32);

        // Update the slot lookups for the swapped value.
        if value_index != last_value_index {
            let last_slot_index = self.get_slot(last_value_index);
            self.set_slot(value_index, last_slot_index);
            slots
                .get_mut(last_slot_index)
                .unwrap()
                .update_value_index(value_index);
        }

        let value_index = value_index.0 as usize;
        assert!(value_index < self.len);

        unsafe {
            let ptr = self.values_ptr.as_ptr();
            self.len -= 1;

            let value = std::ptr::read(ptr.add(value_index));
            std::ptr::copy(
                ptr.add(last_value_index.0 as usize),
                ptr.add(value_index),
                1,
            );

            value
        }
    }

    /// Retreive a reference to the value at `value_index`
    /// Panics if `value_index` is out of bounds
    #[inline(always)]
    fn get(&self, value_index: ValueIndex) -> &T {
        let value_index = value_index.0 as usize;
        assert!(value_index < self.len);
        let ptr = self.values_ptr.as_ptr();
        unsafe { ptr.add(value_index).as_ref().unwrap() }
    }

    /// Retreive a mutable reference to the value at `value_index`
    /// Panics if `value_index` is out of bounds
    #[inline(always)]
    fn get_mut(&mut self, value_index: ValueIndex) -> &mut T {
        let value_index = value_index.0 as usize;
        assert!(value_index < self.len);
        let ptr = self.values_ptr.as_ptr();
        unsafe { ptr.add(value_index).as_mut().unwrap() }
    }

    /// Expands the values region by a fixed amount.
    #[cold]
    fn grow(&mut self) {
        let new_cap = std::cmp::min(self.cap + 1024, MAX_CAP);
        let grow_region = new_cap - self.cap;

        if grow_region > 0 {
            unsafe {
                virtual_commit(
                    self.values_ptr.as_ptr().add(self.len) as _,
                    grow_region * size_of::<T>(),
                );
                virtual_commit(
                    self.slots_ptr.as_ptr().add(self.len) as _,
                    grow_region * size_of::<SlotIndex>(),
                );
            }
        }

        self.cap = new_cap;
    }
}

/// A pool for allocating objects of type T and associating them with a POD `Handle`.
///
/// We do a basic attempt to ensure that mixing handles from different pools with either assert or return None. However
/// it's possible that by accident lookup using a handle from another pool will return a valid object. The pool will
/// not have memory unsafety in this case however, as it will only return valid objects from the pool.
pub struct Pool<T> {
    encode_multiplier: u32,
    decode_multiplier: u32,
    free_slots: FreeSlots,
    slots: Slots,
    values: Values<T>,
    mapping_base: *mut std::ffi::c_void,
    mapping_size: usize,
}

impl<T> Pool<T> {
    /// Creates a new pool.
    ///
    /// This will reserve a large amount of virtual memory for the maximum size of the pool, but won't commit any of it
    /// until it is required.
    pub fn new() -> Self {
        let mut mapping_size = 0;

        let free_slots_offset = mapping_size;
        mapping_size += MAX_IDX * size_of::<u32>();
        mapping_size = align_offset(mapping_size, PAGE_SIZE);

        let slots_offset = mapping_size;
        mapping_size += MAX_IDX * size_of::<Slot>();
        mapping_size = align_offset(mapping_size, PAGE_SIZE);

        let value_slots_offset = mapping_size;
        mapping_size += MAX_CAP * size_of::<u32>();
        mapping_size = align_offset(mapping_size, PAGE_SIZE);

        let values_offset = mapping_size;
        mapping_size += MAX_CAP * size_of::<T>();
        mapping_size = align_offset(mapping_size, PAGE_SIZE);

        let mapping_base = virtual_reserve(mapping_size).expect("failed to map memory");
        let free_slots = unsafe { mapping_base.add(free_slots_offset) } as _;
        let slots = unsafe { mapping_base.add(slots_offset) } as _;
        let value_slots = unsafe { mapping_base.add(value_slots_offset) } as _;
        let values = unsafe { mapping_base.add(values_offset) } as _;

        // virtual reservations are page aligned, so shift out the zeroes in the bottom of the base address.
        let encode_multiplier = mapping_base as usize >> 12;
        // multiplier must be odd to calculate the mod inverse.
        let encode_multiplier = encode_multiplier as u32 | 1;
        let decode_multiplier = mod_inverse_u32(encode_multiplier);

        Self {
            encode_multiplier,
            decode_multiplier,
            free_slots: FreeSlots::new(NonNull::new(free_slots).unwrap()),
            slots: Slots::new(NonNull::new(slots).unwrap()),
            values: Values::new(
                NonNull::new(value_slots).unwrap(),
                NonNull::new(values).unwrap(),
            ),
            mapping_base,
            mapping_size,
        }
    }

    /// Returns the number of values in the pool.
    pub fn len(&self) -> usize {
        self.values.len
    }

    /// Returns `true` if the pool contains no values.
    pub fn is_empty(&self) -> bool {
        self.values.len == 0
    }

    /// Returns a slice containing all values in the pool.
    pub fn values(&self) -> &[T] {
        self.values.as_slice()
    }

    /// Returns a mutable slice containing all values in the pool.
    pub fn values_mut(&mut self) -> &mut [T] {
        self.values.as_mut_slice()
    }

    /// Inserts a value into the pool, returning a handle that represents it.
    #[must_use]
    pub fn insert(&mut self, value: T) -> Handle {
        let value_index = self.values.push(value);

        if self.free_slots.len() < MIN_FREE_SLOTS {
            // We need to grow the slots array if there are insufficient free slots.
            // This is a no-op if we're already at the max capacity of the pool, which weakens the use-after-free
            // detection.
            if let Some((lo, hi)) = self.slots.try_grow() {
                for free_slot_index in lo..hi {
                    self.free_slots.push(SlotIndex(free_slot_index));
                }
            }
        }

        let slot_index = self.free_slots.pop().expect("pool capacity exceeded");
        self.values.set_slot(value_index, slot_index);

        let slot = self.slots.get_mut(slot_index).unwrap();
        slot.set_value_index(value_index);
        Handle::encode(self.encode_multiplier, slot.generation(), slot_index)
    }

    /// Removes a value from the pool, returning the value associated with the handle if it was previously valid.
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let (generation, slot_index) = handle.decode(self.decode_multiplier);

        if let Some(slot) = self.slots.get_mut(slot_index) {
            if slot.generation() == generation {
                self.free_slots.push(slot_index);
                let value_index = slot.value_index();
                slot.set_value_index(ValueIndex::invalid());
                return Some(self.values.swap_remove(value_index, &mut self.slots));
            }
        }

        None
    }

    /// Returns a mutable reference to the value corresponding to the handle.
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let (generation, slot_index) = handle.decode(self.decode_multiplier);

        if let Some(slot) = self.slots.get(slot_index) {
            if slot.generation() == generation {
                return Some(self.values.get_mut(slot.value_index()));
            }
        }

        None
    }

    /// Returns a reference to the value corresponding to the handle.
    pub fn get(&self, handle: Handle) -> Option<&T> {
        let (generation, slot_index) = handle.decode(self.decode_multiplier);

        if let Some(slot) = self.slots.get(slot_index) {
            if slot.generation() == generation {
                return Some(self.values.get(slot.value_index()));
            }
        }

        None
    }

    /// Clears the pool, removing all values without dropping them.
    ///
    /// Does not release any memory.
    fn clear_no_drop(&mut self) {
        for i in 0..self.slots.len {
            let slot_index = SlotIndex(i as u32);
            let slot = self.slots.get_mut(slot_index).unwrap();
            slot.clear_value_index();
            self.free_slots.push(slot_index);
        }
    }

    /// Clears the pool, removing all values.
    ///
    /// Does not release any memory.
    pub fn clear(&mut self) {
        self.clear_no_drop();
        let len = self.values.len;
        self.values.len = 0;
        let to_drop = std::ptr::slice_from_raw_parts_mut(self.values.values_ptr.as_ptr(), len);
        unsafe { std::ptr::drop_in_place(to_drop) };
    }
}

impl<T> Drop for Pool<T> {
    fn drop(&mut self) {
        unsafe {
            let to_drop = std::ptr::slice_from_raw_parts_mut(
                self.values.values_ptr.as_ptr(),
                self.values.len,
            );
            std::ptr::drop_in_place(to_drop);
            virtual_free(self.mapping_base, self.mapping_size).expect("failed to unmap memory");
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::{Handle, Pool, MAX_CAP};

    #[test]
    fn lookup_null() {
        let mut pool = Pool::new();
        assert!(pool.get(Handle::null()).is_none());
        let _ = pool.insert(0);
        assert!(pool.get(Handle::null()).is_none());
    }

    #[test]
    fn insert_lookup_remove() {
        let mut pool = Pool::new();
        assert_eq!(pool.get(Handle::null()), None);

        let handles: [Handle; 500] = std::array::from_fn(|i| pool.insert(i));
        for _ in 0..20 {
            let handles = (0..300_000).map(|i| pool.insert(i)).collect::<Vec<_>>();
            for (i, &handle) in handles.iter().enumerate() {
                assert_eq!(pool.get(handle), Some(&i));
                assert_eq!(pool.remove(handle), Some(i));
                assert_eq!(pool.get(handle), None);
                assert_eq!(pool.remove(handle), None);
            }
        }
        for (i, &handle) in handles.iter().enumerate() {
            assert_eq!(pool.get(handle), Some(&i));
            assert_eq!(pool.remove(handle), Some(i));
            assert_eq!(pool.get(handle), None);
            assert_eq!(pool.remove(handle), None);
        }

        assert_eq!(pool.get(Handle::null()), None);
    }

    // This test is based on randomness in the base address of the pool so disable it by default to
    // avoid flaky tests in CI.
    // We do a basic attempt to ensure that mixing handles from different pools with either assert or return None.
    #[test]
    #[ignore]
    #[should_panic]
    fn test_pool_randomiser() {
        let mut pool_1 = Pool::new();
        let mut pool_2 = Pool::new();
        let handle_1 = pool_1.insert(1);
        let handle_2 = pool_2.insert(1);
        assert_ne!(handle_1, handle_2);
        assert_eq!(pool_1.get(handle_2), None);
        assert_eq!(pool_2.get(handle_1), None);
    }

    #[test]
    fn capacity() {
        let mut pool = Pool::new();

        let handles = (0..MAX_CAP).map(|i| pool.insert(i)).collect::<Vec<_>>();

        assert_eq!(pool.len(), MAX_CAP);

        for (i, &handle) in handles.iter().enumerate() {
            assert_eq!(pool.get(handle), Some(&i));
        }

        for (i, &handle) in handles.iter().enumerate() {
            assert_eq!(pool.remove(handle), Some(i));
        }

        assert!(pool.is_empty());
    }

    #[test]
    fn use_after_free() {
        let mut pool = Pool::new();

        let handle = pool.insert(1);
        assert_eq!(pool.remove(handle), Some(1));

        for _ in 0..65536 {
            let new_handle = pool.insert(1);
            assert_eq!(pool.remove(new_handle), Some(1));
            assert_ne!(handle, new_handle);
            assert_eq!(pool.get(handle), None);
        }
    }

    #[test]
    fn drop_it_like_its_hot() {
        static DROP_COUNT: AtomicU32 = AtomicU32::new(0);
        struct Snoop {}
        impl Drop for Snoop {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
        let mut pool = Pool::new();

        let _ = pool.insert(Snoop {});
        let _ = pool.insert(Snoop {});
        let handle = pool.insert(Snoop {});

        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 0);
        pool.remove(handle);
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
        pool.clear();
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 3);

        let _ = pool.insert(Snoop {});
        drop(pool);
        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 4);
    }
}
