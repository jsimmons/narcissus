use std::{
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicI32, Ordering},
};

struct Inner<T: ?Sized> {
    // Number of strong references in addition to the current value.
    //
    // A negative value indicates a non-atomic reference count, counting up from
    // `i32::MIN`
    //
    // A positive value indicates an atomic reference count, counting up from `0`
    strong: AtomicI32,
    value: T,
}

impl<T> Inner<T> {
    #[inline]
    fn new(value: T) -> Self {
        Self {
            strong: AtomicI32::new(i32::MIN + 1),
            value,
        }
    }

    #[inline]
    fn new_atomic(value: T) -> Self {
        Self {
            strong: AtomicI32::new(1),
            value,
        }
    }
}

impl<T: ?Sized> Inner<T> {
    #[inline]
    fn incr_strong(&self) {
        let strong = self.strong.load(Ordering::Relaxed);
        if strong < 0 {
            self.strong.store(strong.wrapping_add(1), Ordering::Relaxed);
        } else {
            self.strong.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline]
    fn decr_strong(&self) -> bool {
        let strong = self.strong.load(Ordering::Relaxed);
        if strong < 0 {
            self.strong.store(strong.wrapping_sub(1), Ordering::Release);
            strong != i32::MIN + 1
        } else {
            let strong = self.strong.fetch_sub(1, Ordering::Release);
            strong != 1
        }
    }

    #[inline]
    fn incr_strong_atomic(&self) {
        self.strong.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn decr_strong_atomic(&self) -> bool {
        self.strong.fetch_sub(1, Ordering::Release) != 1
    }

    #[inline]
    fn upgrade(&self) {
        let strong = self.strong.load(Ordering::Relaxed);
        if strong < 0 {
            self.strong
                .store(strong.wrapping_add(i32::MIN), Ordering::Relaxed);
        }
    }
}

pub struct Rc<T: ?Sized> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        Self::from_inner(Box::leak(Box::new(Inner::new(value))).into())
    }
}

impl<T: Default> Default for Rc<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized> Rc<T> {
    #[inline]
    pub fn strong_count(&self) -> i32 {
        let strong = self.inner().strong.load(Ordering::Relaxed);
        if strong < 0 {
            strong.wrapping_add(i32::MIN)
        } else {
            strong
        }
    }

    #[inline]
    pub fn is_unique(&mut self) -> bool {
        let strong = self.inner().strong.load(Ordering::Relaxed);
        strong == 1 || strong == i32::MIN + 1
    }

    #[inline]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_unique() {
            // This unsafety is ok because we're guaranteed that the pointer
            // returned is the *only* pointer that will ever be returned to T. Our
            // reference count is guaranteed to be 1 at this point, and we required
            // the Arc itself to be `mut`, so we're returning the only possible
            // reference to the inner data.
            Some(unsafe { self.get_mut_unchecked() })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// Any other [`Rc`] or [`Arc`] pointers to the same allocation must not be
    /// dereferenced for the duration of the returned borrow. This is trivially the
    /// case if no such pointers exist, for example immediately after [`Arc::new`].
    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T { unsafe {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts.
        &mut (*self.ptr.as_ptr()).value
    }}

    #[inline]
    fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[cold]
    #[inline(never)]
    fn drop_slow(&self) {
        std::sync::atomic::fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

impl<T: ?Sized> Clone for Rc<T> {
    fn clone(&self) -> Self {
        self.inner().incr_strong();
        Self::from_inner(self.inner().into())
    }
}

impl<T: ?Sized> Drop for Rc<T> {
    fn drop(&mut self) {
        if !self.inner().decr_strong() {
            self.drop_slow();
        }
    }
}

impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    // Inner is valid whilever we have a valid Rc.
    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

pub struct Arc<T: ?Sized> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    pub fn new(value: T) -> Self {
        Self::from_inner(Box::leak(Box::new(Inner::new_atomic(value))).into())
    }
}

impl<T: Default> Default for Arc<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized> Arc<T> {
    pub fn from_rc(rc: &Rc<T>) -> Self {
        let inner = rc.inner();
        inner.upgrade();
        inner.incr_strong();
        Self::from_inner(inner.into())
    }

    #[inline]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }

    #[inline]
    pub fn strong_count(&self) -> i32 {
        let strong = self.inner().strong.load(Ordering::Relaxed);
        if strong < 0 {
            strong.wrapping_add(i32::MIN)
        } else {
            strong
        }
    }

    #[inline]
    pub fn is_unique(&self) -> bool {
        let strong = self.inner().strong.load(Ordering::Acquire);
        strong == 1 || strong == i32::MIN + 1
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_unique() {
            // SAFETY: We're guaranteed that the pointer returned is the *only* pointer that
            // will ever be returned to T because our reference count is 1, and we required
            // the Arc reference itself to be mutable.
            Some(unsafe { self.get_mut_unchecked() })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// Any other [`Rc`] or [`Arc`] pointers to the same allocation must not be dereferenced for the duration of the
    /// returned borrow. This is trivially the case if no such pointers exist, for example immediately after
    /// [`Arc::new`].
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T { unsafe {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts.
        &mut (*self.ptr.as_ptr()).value
    }}

    fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn inner(&self) -> &Inner<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[cold]
    #[inline(never)]
    fn drop_slow(&self) {
        std::sync::atomic::fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    fn clone(&self) -> Self {
        self.inner().incr_strong_atomic();
        Self::from_inner(self.inner().into())
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    fn drop(&mut self) {
        if !self.inner().decr_strong_atomic() {
            self.drop_slow()
        }
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;

    // Inner is value whilever we have a valid Arc.
    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn rc_drop() {
        use std::sync::atomic::{AtomicU32, Ordering};

        struct A<'a>(&'a AtomicU32);
        impl Drop for A<'_> {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let counter = AtomicU32::new(0);
        let a = Arc::new(A(&counter));
        assert_eq!(counter.load(Ordering::Relaxed), 0);
        let b = a.clone();
        assert_eq!(counter.load(Ordering::Relaxed), 0);
        drop(a);
        assert_eq!(counter.load(Ordering::Relaxed), 0);
        drop(b);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn rc_double_upgrade() {
        let rc1 = Rc::new(());
        assert_eq!(rc1.strong_count(), 1);
        let _rc2 = rc1.clone();
        assert_eq!(rc1.strong_count(), 2);
        let _arc1 = Arc::from_rc(&rc1);
        assert_eq!(rc1.strong_count(), 3);
        let _arc2 = Arc::from_rc(&rc1);
        assert_eq!(rc1.strong_count(), 4);
    }
}
