use core::{
    cell::{Cell, UnsafeCell},
    future::Future,
    hint,
    ops::{self, Deref as _},
    pin::Pin,
    ptr::NonNull,
    task::{Context, Poll, Waker},
};

use pin_project::{pin_project, pinned_drop};

use crate::dll::{DoublyLinkedList, Link};

/// `async`-aware `Mutex`
pub struct Mutex<T> {
    data: UnsafeCell<T>,
    locked: Cell<bool>,
    // linked list of pinned wakers
    wakers: DoublyLinkedList<Cell<Option<Waker>>>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            locked: Cell::new(false),
            wakers: DoublyLinkedList::new(),
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
        #[pin_project(PinnedDrop)]
        struct Lock<'a, T> {
            #[pin]
            link: Option<Link<Cell<Option<Waker>>>>,
            mutex: &'a Mutex<T>,
        }

        impl<'m, T> Future for Lock<'m, T> {
            type Output = MutexGuard<'m, T>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut self_ = self.project();

                if let Some(guard) = self_.mutex.try_lock() {
                    Poll::Ready(guard)
                } else {
                    if self_.link.is_none() {
                        // pin the waker by storing in the already pinned `Lock` instance
                        self_
                            .link
                            .set(Some(Link::new(Cell::new(Some(cx.waker().clone())))));

                        unsafe {
                            self_.mutex.wakers.push_front(
                                self_
                                    .link
                                    .as_ref() // Pin<&Option<Link>>
                                    .deref() // &Option<Link>
                                    .as_ref() // Option<&Link>
                                    // always in the `Some` variant due to the previous operation
                                    .unwrap_or_else(|| hint::unreachable_unchecked()),
                            );
                        }
                    }

                    Poll::Pending
                }
            }
        }

        #[pinned_drop]
        impl<T> PinnedDrop for Lock<'_, T> {
            fn drop(self: Pin<&mut Self>) {
                let self_ = self.project();

                if let Some(link) = self_.link.as_ref().deref().as_ref() {
                    unsafe {
                        self_.mutex.wakers.unlink(NonNull::from(link));
                    }

                    // destroy the `Waker` if it was not used
                    drop(link.data().take());
                }
            }
        }

        Lock {
            link: None,
            mutex: self,
        }
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

        // wake exactly one task
        while let Some(link) = self.mutex.wakers.pop_front() {
            if let Some(waker) = unsafe { link.as_ref().data().take() } {
                waker.wake();
                return;
            }
        }
    }
}
