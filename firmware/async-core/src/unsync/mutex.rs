use core::{
    cell::{Cell, UnsafeCell},
    future::Future,
    ops,
    pin::Pin,
    task::{Context, Poll},
};

/// `async`-aware `Mutex`
pub struct Mutex<T> {
    data: UnsafeCell<T>,
    locked: Cell<bool>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            locked: Cell::new(false),
        }
    }

    /// Attempts to acquire the lock
    pub fn try_lock<'m>(&'m self) -> Option<MutexGuard<'m, T>> {
        if self.locked.get() {
            None
        } else {
            self.locked.set(true);
            Some(MutexGuard { mutex: self })
        }
    }

    /// Acquires the lock
    pub fn lock<'m>(&'m self) -> impl Future<Output = MutexGuard<'m, T>> {
        struct Lock<'a, T> {
            mutex: &'a Mutex<T>,
        }

        impl<'m, T> Future for Lock<'m, T> {
            type Output = MutexGuard<'m, T>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                if let Some(guard) = self.mutex.try_lock() {
                    Poll::Ready(guard)
                } else {
                    Poll::Pending
                }
            }
        }

        Lock { mutex: self }
    }
}

/// A locked mutex
pub struct MutexGuard<'m, T> {
    mutex: &'m Mutex<T>,
}

impl<T> ops::Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.set(false);

        // wake up a task waiting to claim this mutex
        asm::sev();
    }
}
