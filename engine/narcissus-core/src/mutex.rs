use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32, Ordering},
};

use crate::{waiter, PhantomUnsend};

#[cfg(debug_assertions)]
#[inline(always)]
fn thread_id() -> std::thread::ThreadId {
    std::thread::current().id()
}

const UNLOCKED: i32 = 0;
const LOCKED: i32 = 1;
const LOCKED_WAIT: i32 = 2;

pub struct Mutex<T: ?Sized> {
    #[cfg(debug_assertions)]
    thread_id: std::cell::Cell<Option<std::thread::ThreadId>>,
    control: AtomicI32,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    mutex: &'a Mutex<T>,
    phantom: PhantomUnsend,
}

impl<'a, T: ?Sized + 'a> MutexGuard<'a, T> {
    pub fn new(mutex: &'a Mutex<T>) -> Self {
        MutexGuard {
            mutex,
            phantom: PhantomUnsend {},
        }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { self.mutex.raw_unlock() }
    }
}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            control: AtomicI32::new(UNLOCKED),
            #[cfg(debug_assertions)]
            thread_id: std::cell::Cell::new(None),
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // SAFETY: `raw_lock()` will deadlock if recursive acquisition is attempted, so the
        // following sequence cannot generate multiple mutable references.
        // ```
        //   let mutex = Mutex::new(1);
        //   let mut lock1 = mutex.lock();
        //   let mut lock2 = mutex.lock();
        //   let a = &mut *lock1;
        //   let b = &mut *lock2;
        // ```
        // In a debug configuration it will assert instead.
        unsafe {
            self.raw_lock();
            MutexGuard::new(self)
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        unsafe {
            if self.raw_try_lock() {
                Some(MutexGuard::new(self))
            } else {
                None
            }
        }
    }

    pub fn unlock(guard: MutexGuard<'_, T>) {
        drop(guard)
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    unsafe fn raw_lock(&self) {
        #[cfg(debug_assertions)]
        if self.thread_id.get() == Some(std::thread::current().id()) {
            panic!("recursion not supported")
        }

        let mut c = self.control.load(Ordering::Relaxed);
        if c == UNLOCKED {
            match self.control.compare_exchange_weak(
                UNLOCKED,
                LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    self.thread_id.set(Some(thread_id()));
                    return;
                }
                Err(x) => c = x,
            }
        }

        loop {
            if c != LOCKED_WAIT {
                match self.control.compare_exchange_weak(
                    LOCKED,
                    LOCKED_WAIT,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(x) => c = x,
                    Err(x) => c = x,
                }
            }

            if c == LOCKED_WAIT {
                waiter::wait(&self.control, LOCKED_WAIT, None);
                c = self.control.load(Ordering::Relaxed);
            }

            if c == UNLOCKED {
                match self.control.compare_exchange_weak(
                    UNLOCKED,
                    LOCKED_WAIT,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        self.thread_id.set(Some(thread_id()));
                        return;
                    }
                    Err(x) => c = x,
                }
            }
        }
    }

    unsafe fn raw_try_lock(&self) -> bool {
        #[cfg(debug_assertions)]
        if self.thread_id.get() == Some(thread_id()) {
            panic!("recursion not supported")
        }

        if self.control.load(Ordering::Relaxed) == UNLOCKED
            && self
                .control
                .compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
        {
            #[cfg(debug_assertions)]
            self.thread_id.set(Some(thread_id()));
            true
        } else {
            false
        }
    }

    unsafe fn raw_unlock(&self) {
        #[cfg(debug_assertions)]
        self.thread_id.set(None);

        if self.control.fetch_sub(1, Ordering::Release) != LOCKED {
            self.control.store(UNLOCKED, Ordering::Release);
            waiter::wake_n(&self.control, 1);
        }
    }
}

impl<T: ?Sized + Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn lock_unlock() {
        let mutex = Mutex::new(10);
        {
            let lock = mutex.lock();
            assert_eq!(*lock, 10);
        }
    }

    #[test]
    fn mutual_exclusion() {
        let barrier = Arc::new(std::sync::Barrier::new(8));
        let mut value = Arc::new(Mutex::new(0));
        let mut threads = (0..8)
            .map(|_| {
                let barrier = barrier.clone();
                let value = value.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    for _ in 0..100_000 {
                        *value.lock() += 1;
                    }
                })
            })
            .collect::<Vec<_>>();

        for thread in threads.drain(..) {
            thread.join().unwrap();
        }

        let value = *value.get_mut().unwrap().get_mut();
        assert_eq!(value, 800_000);
    }

    // This test will deadlock in release builds, so don't run it.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "recursion not supported")]
    fn recursion() {
        let mutex = Mutex::new(1);
        let mut lock1 = mutex.lock();
        let mut lock2 = mutex.lock();
        // Summon Cthulhu
        let _a = &mut *lock1;
        let _b = &mut *lock2;
    }

    // This test will deadlock in release builds, so don't run it.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "recursion not supported")]
    fn recursion_try() {
        let mutex = Mutex::new(1);
        let mut lock1;
        loop {
            if let Some(lock) = mutex.try_lock() {
                lock1 = lock;
                break;
            }
        }

        let mut lock2;
        loop {
            if let Some(lock) = mutex.try_lock() {
                lock2 = lock;
                break;
            }
        }

        // Summon Cthulhu
        let _a = &mut *lock1;
        let _b = &mut *lock2;
    }
}
