#![allow(unused)]

use std::{
    iter::FusedIterator,
    slice::{Iter, IterMut},
};

/// Shim implementation of some unstable Rust stdlib functions.

/// An iterator over a slice in (non-overlapping) chunks (`N` elements at a
/// time), starting at the beginning of the slice.
///
/// When the slice len is not evenly divided by the chunk size, the last
/// up to `N-1` elements will be omitted but can be retrieved from
/// the [`remainder`] function from the iterator.
///
/// This struct is created by the [`array_chunks`] method on [slices].
///
/// [`array_chunks`]: slice::array_chunks
/// [`remainder`]: ArrayChunks::remainder
/// [slices]: slice
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArrayChunks<'a, T: 'a, const N: usize> {
    iter: Iter<'a, [T; N]>,
    rem: &'a [T],
}

impl<'a, T, const N: usize> ArrayChunks<'a, T, N> {
    #[inline]
    pub(super) fn new(slice: &'a [T]) -> Self {
        let (array_slice, rem) = as_chunks(slice);
        Self {
            iter: array_slice.iter(),
            rem,
        }
    }

    /// Returns the remainder of the original slice that is not going to be
    /// returned by the iterator. The returned slice has at most `N-1`
    /// elements.
    #[must_use]
    pub fn remainder(&self) -> &'a [T] {
        self.rem
    }
}

// FIXME(#26925) Remove in favor of `#[derive(Clone)]`
impl<T, const N: usize> Clone for ArrayChunks<'_, T, N> {
    fn clone(&self) -> Self {
        ArrayChunks {
            iter: self.iter.clone(),
            rem: self.rem,
        }
    }
}

impl<'a, T, const N: usize> Iterator for ArrayChunks<'a, T, N> {
    type Item = &'a [T; N];

    #[inline]
    fn next(&mut self) -> Option<&'a [T; N]> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for ArrayChunks<'a, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [T; N]> {
        self.iter.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}

impl<T, const N: usize> ExactSizeIterator for ArrayChunks<'_, T, N> {}

impl<T, const N: usize> FusedIterator for ArrayChunks<'_, T, N> {}

/// An iterator over a slice in (non-overlapping) mutable chunks (`N` elements
/// at a time), starting at the beginning of the slice.
///
/// When the slice len is not evenly divided by the chunk size, the last
/// up to `N-1` elements will be omitted but can be retrieved from
/// the [`into_remainder`] function from the iterator.
///
/// This struct is created by the [`array_chunks_mut`] method on [slices].f
///
/// [`array_chunks_mut`]: slice::array_chunks_mut
/// [`into_remainder`]: ../../std/slice/struct.ArrayChunksMut.html#method.into_remainder
/// [slices]: slice
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArrayChunksMut<'a, T: 'a, const N: usize> {
    iter: IterMut<'a, [T; N]>,
    rem: &'a mut [T],
}

impl<'a, T, const N: usize> ArrayChunksMut<'a, T, N> {
    #[inline]
    pub(super) fn new(slice: &'a mut [T]) -> Self {
        let (array_slice, rem) = as_chunks_mut(slice);
        Self {
            iter: array_slice.iter_mut(),
            rem,
        }
    }

    /// Returns the remainder of the original slice that is not going to be
    /// returned by the iterator. The returned slice has at most `N-1`
    /// elements.
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_remainder(self) -> &'a mut [T] {
        self.rem
    }
}

impl<'a, T, const N: usize> Iterator for ArrayChunksMut<'a, T, N> {
    type Item = &'a mut [T; N];

    #[inline]
    fn next(&mut self) -> Option<&'a mut [T; N]> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for ArrayChunksMut<'a, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut [T; N]> {
        self.iter.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}

impl<T, const N: usize> ExactSizeIterator for ArrayChunksMut<'_, T, N> {}

impl<T, const N: usize> FusedIterator for ArrayChunksMut<'_, T, N> {}

/// Returns an iterator over `N` elements of the slice at a time, starting at the
/// beginning of the slice.
///
/// The chunks are array references and do not overlap. If `N` does not divide the
/// length of the slice, then the last up to `N-1` elements will be omitted and can be
/// retrieved from the `remainder` function of the iterator.
///
/// This method is the const generic equivalent of [`chunks_exact`].
///
/// # Panics
///
/// Panics if `N` is 0. This check will most probably get changed to a compile time
/// error before this method gets stabilized.
///
/// [`chunks_exact`]: slice::chunks_exact
#[inline]
pub fn array_chunks<T, const N: usize>(slice: &[T]) -> ArrayChunks<'_, T, N> {
    assert_ne!(N, 0);
    ArrayChunks::new(slice)
}

/// Returns an iterator over `N` elements of the slice at a time, starting at the
/// beginning of the slice.
///
/// The chunks are mutable array references and do not overlap. If `N` does not divide
/// the length of the slice, then the last up to `N-1` elements will be omitted and
/// can be retrieved from the `into_remainder` function of the iterator.
///
/// This method is the const generic equivalent of [`chunks_exact_mut`].
///
/// # Panics
///
/// Panics if `N` is 0. This check will most probably get changed to a compile time
/// error before this method gets stabilized.
///
/// [`chunks_exact_mut`]: slice::chunks_exact_mut
#[inline]
pub fn array_chunks_mut<T, const N: usize>(slice: &mut [T]) -> ArrayChunksMut<'_, T, N> {
    assert_ne!(N, 0);
    ArrayChunksMut::new(slice)
}

/// Splits the slice into a slice of `N`-element arrays,
/// assuming that there's no remainder.
///
/// # Safety
///
/// This may only be called when
/// - The slice splits exactly into `N`-element chunks (aka `self.len() % N == 0`).
/// - `N != 0`.
///
/// // These would be unsound:
/// // let chunks: &[[_; 5]] = slice.as_chunks_unchecked() // The slice length is not a multiple of 5
/// // let chunks: &[[_; 0]] = slice.as_chunks_unchecked() // Zero-length chunks are never allowed
/// ```
#[inline]
#[must_use]
pub unsafe fn as_chunks_unchecked<T, const N: usize>(slice: &[T]) -> &[[T; N]] {
    // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
    let new_len = slice.len() / N;
    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast(), new_len) }
}

/// Splits the slice into a slice of `N`-element arrays,
/// starting at the beginning of the slice,
/// and a remainder slice with length strictly less than `N`.
///
/// # Panics
///
/// Panics if `N` is 0. This check will most probably get changed to a compile time
/// error before this method gets stabilized.
///
#[inline]
#[must_use]
pub fn as_chunks<T, const N: usize>(slice: &[T]) -> (&[[T; N]], &[T]) {
    assert_ne!(N, 0);
    let len = slice.len() / N;
    let (multiple_of_n, remainder) = slice.split_at(len * N);
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N.
    let array_slice = unsafe { as_chunks_unchecked(multiple_of_n) };
    (array_slice, remainder)
}

/// Splits the slice into a slice of `N`-element arrays,
/// assuming that there's no remainder.
///
/// # Safety
///
/// This may only be called when
/// - The slice splits exactly into `N`-element chunks (aka `self.len() % N == 0`).
/// - `N != 0`.
///
/// // These would be unsound:
/// // let chunks: &[[_; 5]] = slice.as_chunks_unchecked_mut() // The slice length is not a multiple of 5
/// // let chunks: &[[_; 0]] = slice.as_chunks_unchecked_mut() // Zero-length chunks are never allowed
/// ```
#[inline]
#[must_use]
pub unsafe fn as_chunks_unchecked_mut<T, const N: usize>(slice: &mut [T]) -> &mut [[T; N]] {
    let this = &*slice;
    // SAFETY: Caller must guarantee that `N` is nonzero and exactly divides the slice length
    let new_len = this.len() / N;
    // SAFETY: We cast a slice of `new_len * N` elements into
    // a slice of `new_len` many `N` elements chunks.
    unsafe { std::slice::from_raw_parts_mut(slice.as_mut_ptr().cast(), new_len) }
}

/// Splits the slice into a slice of `N`-element arrays,
/// starting at the beginning of the slice,
/// and a remainder slice with length strictly less than `N`.
///
/// # Panics
///
/// Panics if `N` is 0. This check will most probably get changed to a compile time
/// error before this method gets stabilized.
/// ```
#[inline]
#[must_use]
pub fn as_chunks_mut<T, const N: usize>(slice: &mut [T]) -> (&mut [[T; N]], &mut [T]) {
    assert_ne!(N, 0);
    let len = slice.len() / N;
    let (multiple_of_n, remainder) = slice.split_at_mut(len * N);
    // SAFETY: We already panicked for zero, and ensured by construction
    // that the length of the subslice is a multiple of N.
    let array_slice = unsafe { as_chunks_unchecked_mut(multiple_of_n) };
    (array_slice, remainder)
}

/// Divides one slice into an array and a remainder slice at an index.
///
/// The array will contain all indices from `[0, N)` (excluding
/// the index `N` itself) and the slice will contain all
/// indices from `[N, len)` (excluding the index `len` itself).
///
/// # Panics
///
/// Panics if `N > len`.
/// ```
#[inline]
#[track_caller]
#[must_use]
pub fn split_array_ref<T, const N: usize>(slice: &[T]) -> (&[T; N], &[T]) {
    let (a, b) = slice.split_at(N);
    // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at)
    unsafe { (&*(a.as_ptr() as *const [T; N]), b) }
}

/// Divides one mutable slice into an array and a remainder slice at an index.
///
/// The array will contain all indices from `[0, N)` (excluding
/// the index `N` itself) and the slice will contain all
/// indices from `[N, len)` (excluding the index `len` itself).
///
/// # Panics
///
/// Panics if `N > len`.
///
#[inline]
#[track_caller]
#[must_use]
pub fn split_array_mut<T, const N: usize>(slice: &mut [T]) -> (&mut [T; N], &mut [T]) {
    let (a, b) = slice.split_at_mut(N);
    // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at_mut)
    unsafe { (&mut *(a.as_mut_ptr() as *mut [T; N]), b) }
}
