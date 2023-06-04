use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Release<T> {
    /// There are other outstanding references to this object.
    Shared,
    /// This was the final reference, contains the object the container was previously holding.
    Unique(T),
}

struct Inner<T> {
    strong: AtomicU32,
    value: ManuallyDrop<T>,
}

impl<T> Inner<T> {
    #[inline]
    fn new(value: T) -> Self {
        Self {
            strong: AtomicU32::new(1),
            value: ManuallyDrop::new(value),
        }
    }
}

impl<T> Inner<T> {
    #[inline]
    fn incr_strong(&self) {
        self.strong.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn decr_strong(&self) -> bool {
        self.strong.fetch_sub(1, Ordering::Release) != 1
    }
}

/// A thread-safe reference-counting pointer with manual management.
///
/// The type [`ManualArc<T>`] provides shared ownership of a value of type T, allocated in the heap.
/// Invoking clone on [`ManualArc`] produces a new [`ManualArc`] instance, which points to the same
/// allocation as the source, while increasing a reference count.
///
/// Before a [`ManualArc`] is dropped, the [`ManualArc::release`] method must be called.
/// [`ManualArc::release`] will return the variant [`Release::Shared`] if there are other references
/// outstanding, or [`Release::Unique`] with the contained object if the release operation removes
/// the final reference.
pub struct ManualArc<T> {
    ptr: Option<NonNull<Inner<T>>>,
    phantom: PhantomData<Inner<T>>,
}

impl<T> ManualArc<T> {
    pub fn new(value: T) -> Self {
        Self::from_inner(Box::leak(Box::new(Inner::new(value))).into())
    }

    #[inline]
    fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
        Self {
            ptr: Some(ptr),
            phantom: PhantomData,
        }
    }

    /// Consumes `self`, decrementing the reference count.
    ///
    /// Returns the variant [`Release::Shared`] if there are other [`ManualArc`] instances still
    /// holding references to the same object, or [`Release::Unique`] with the previously contained
    /// object if the release operation removes the last reference.
    ///
    /// Must be explicitly called to drop [`ManualArc`] instances. Dropping implicitly or explicitly
    /// without calling this function will panic.
    pub fn release(mut self) -> Release<T> {
        #[cold]
        #[inline(never)]
        unsafe fn release_slow<T>(ptr: NonNull<Inner<T>>) -> T {
            // Ref-counting operations imply a full memory barrier on x86, but not in general. So
            // insert an acquire barrier on the slow path to ensure all modifications to inner are
            // visible before we call drop.
            std::sync::atomic::fence(Ordering::Acquire);

            // SAFETY: Was created by Box::leak in the constructor, so it's valid to recreate a box.
            let mut inner = Box::from_raw(ptr.as_ptr());
            // extract the value from the container so we can return it.
            let value = ManuallyDrop::take(&mut inner.value);
            // since the contained value is wrapped in `ManuallyDrop` it won't be dropped here.
            drop(inner);

            value
        }

        // SAFETY: `release` consumes `self` so it's impossible to call twice on the same instance,
        // release is also the only function able to invalidate the pointer. Hence the pointer is
        // always valid here.
        unsafe {
            // Replace ptr with None so that the drop function doesn't panic
            let ptr = self.ptr.take();
            let ptr = ptr.unwrap_unchecked();
            let inner = ptr.as_ref();
            if inner.decr_strong() {
                Release::Shared
            } else {
                // We have released the last reference to this inner, so we need to free it and
                // return the contained value.
                let value = release_slow(ptr);
                Release::Unique(value)
            }
        }
    }
}

impl<T: Default> Default for ManualArc<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Clone for ManualArc<T> {
    fn clone(&self) -> Self {
        // SAFETY: Inner is valid whilever we have a valid `ManualArc`, and so long as we are outside
        // the `release` function.
        unsafe {
            let ptr = self.ptr.unwrap_unchecked();
            ptr.as_ref().incr_strong();
            Self::from_inner(ptr)
        }
    }
}

impl<T> Drop for ManualArc<T> {
    fn drop(&mut self) {
        if self.ptr.is_some() && !std::thread::panicking() {
            panic!("must call `ManualArc::release` before value is dropped");
        }
    }
}

impl<T> Deref for ManualArc<T> {
    type Target = T;

    // SAFETY: Inner is valid whilever we have a valid `ManualArc`, and so long as we are outside
    // the `release` function.
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.unwrap_unchecked().as_ref() }
            .value
            .deref()
    }
}

#[cfg(test)]
mod tests {
    use super::{ManualArc, Release};

    #[test]
    fn basic() {
        let arc1 = ManualArc::new(42);
        let arc2 = arc1.clone();

        assert_eq!(*arc1, 42);
        assert_eq!(*arc2, 42);

        assert_eq!(arc2.release(), Release::Shared);
        assert_eq!(arc1.release(), Release::Unique(42));
    }

    #[test]
    #[should_panic]
    fn drop_without_release() {
        let arc = ManualArc::new(32);
        drop(arc);
    }
}
