use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Release<T> {
    Shared,
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

pub struct ManualArc<T> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,

    #[cfg(debug_assertions)]
    has_released: bool,
}

impl<T> ManualArc<T> {
    pub fn new(value: T) -> Self {
        Self::from_inner(Box::leak(Box::new(Inner::new(value))).into())
    }

    #[inline]
    fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,

            #[cfg(debug_assertions)]
            has_released: false,
        }
    }

    #[inline]
    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[cold]
    #[inline(never)]
    fn release_slow(&self) -> T {
        std::sync::atomic::fence(Ordering::Acquire);
        let value;
        unsafe {
            let mut inner = Box::from_raw(self.ptr.as_ptr());
            // extract the value from the container.
            value = ManuallyDrop::take(&mut inner.value);
            // since the value is wrapped in `ManuallyDrop` it won't be dropped here.
            drop(inner);
        }
        value
    }

    pub fn release(&mut self) -> Release<T> {
        #[cfg(debug_assertions)]
        {
            assert!(!self.has_released);
            self.has_released = true;
        }

        if self.inner().decr_strong() {
            Release::Shared
        } else {
            Release::Unique(self.release_slow())
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
        self.inner().incr_strong();
        Self::from_inner(self.inner().into())
    }
}

#[cfg(debug_assertions)]
impl<T> Drop for ManualArc<T> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.has_released, "must release manually before drop");
        }
    }
}

impl<T> Deref for ManualArc<T> {
    type Target = T;

    // Inner is valid whilever we have a valid ManualArc.
    fn deref(&self) -> &Self::Target {
        self.inner().value.deref()
    }
}

#[cfg(test)]
mod tests {
    use super::{ManualArc, Release};

    #[test]
    fn basic() {
        let mut arc1 = ManualArc::new(42);
        let mut arc2 = arc1.clone();

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
