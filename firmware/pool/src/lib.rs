//! `async`-aware memory pool

#![no_std]

use core::{
    mem, ops,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

#[doc(hidden)]
pub trait Pool: 'static {
    // keep things simple
    type T: Copy;

    #[doc(hidden)]
    fn get() -> &'static PoolImpl<Self::T>;
}

#[doc(hidden)]
pub struct PoolImpl<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> PoolImpl<T> {
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[doc(hidden)]
    pub fn pop(&self) -> Option<NonNull<Node<T>>> {
        loop {
            let head = self.head.load(Ordering::Relaxed);
            if let Some(nn_head) = NonNull::new(head) {
                let next = unsafe { node_next(head).read() };

                match self.head.compare_exchange_weak(
                    head,
                    next,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break Some(nn_head),
                    // interrupt occurred
                    Err(_) => continue,
                }
            } else {
                // stack is observed as empty
                break None;
            }
        }
    }

    #[doc(hidden)]
    pub unsafe fn push(&self, new_head: NonNull<Node<T>>) {
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            node_next(new_head.as_ptr()).write(head);

            if let Err(p) = self.head.compare_exchange_weak(
                head,
                new_head.as_ptr(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                head = p
            } else {
                // memory block became available: wake up a task
                asm::sev();
                return;
            }
        }
    }
}

/// Declares a memory pool
#[macro_export]
macro_rules! pool {
    ($(#[$attr:meta])* pub $ident:ident : [u8; $N:expr]) => {
        $(#[$attr])*
        pub struct $ident;

        impl $crate::Pool for $ident {
            type T = [u8; $N];

            #[inline(always)]
            fn get() -> &'static $crate::PoolImpl<[u8; $N]> {
                static $ident: $crate::PoolImpl<[u8; $N]> = $crate::PoolImpl::new();
                &$ident
            }
        }

        impl $ident {
            /// The size of the memory blocks managed by this pool
            pub const SIZE: usize = $N;

            /// Tries to acquire a memory block
            #[allow(dead_code)]
            pub fn try_alloc() -> Option<$crate::Box<$ident>> {
                unsafe {
                    <$ident as $crate::Pool>::get()
                        .pop()
                        .map(|n| $crate::Box::from_raw(n.as_ptr()))
                }
            }

            /// Acquires a memory block
            #[allow(dead_code)]
            pub fn alloc() -> impl core::future::Future<Output = $crate::Box<$ident>> {
                struct Alloc;

                impl core::future::Future for Alloc {
                    type Output = $crate::Box<$ident>;

                    fn poll(
                        self: core::pin::Pin<&mut Self>,
                        _: &mut core::task::Context,
                    ) -> core::task::Poll<Self::Output> {
                        $ident::try_alloc()
                            .map(core::task::Poll::Ready)
                            .unwrap_or(core::task::Poll::Pending)
                    }
                }

                Alloc
            }

            /// Gives the pool a memory block to manage
            #[allow(dead_code)]
            pub fn manage(node: &'static mut core::mem::MaybeUninit<$crate::Node<[u8; $N]>>) {
                unsafe {
                    <$ident as $crate::Pool>::get().push(core::ptr::NonNull::from(node).cast())
                }
            }
        }
    };
}

/// Owning pointer
pub struct Box<P>
where
    P: Pool,
{
    node: NonNull<Node<P::T>>,
}

impl<P> Box<P>
where
    P: Pool,
{
    /// Consumes the `Box`, returning a raw pointer
    pub unsafe fn from_raw(raw: *mut Node<P::T>) -> Self {
        Box {
            node: NonNull::new_unchecked(raw),
        }
    }

    /// Consumes the `Box`, returning a raw pointer
    pub fn into_raw(self) -> *mut Node<P::T> {
        let node = self.node;
        mem::forget(self);
        node.as_ptr()
    }
}

unsafe impl<P> Send for Box<P>
where
    P: Pool,
    P::T: Send,
{
}

unsafe impl<P> Sync for Box<P>
where
    P: Pool,
    P::T: Sync,
{
}

impl<P> ops::Deref for Box<P>
where
    P: Pool,
{
    type Target = P::T;

    fn deref(&self) -> &P::T {
        unsafe { &*node_data(self.node.as_ptr()) }
    }
}

impl<P> ops::DerefMut for Box<P>
where
    P: Pool,
{
    fn deref_mut(&mut self) -> &mut P::T {
        unsafe { &mut *node_data(self.node.as_ptr()) }
    }
}

impl<P> Drop for Box<P>
where
    P: Pool,
{
    fn drop(&mut self) {
        unsafe { P::get().push(self.node) }
    }
}

#[cfg_attr(target_pointer_width = "32", repr(align(4)))]
#[cfg_attr(target_pointer_width = "64", repr(align(8)))]
#[repr(C)]
pub struct Node<T> {
    data: T,
}

fn node_data<T>(node: *mut Node<T>) -> *mut T {
    node.cast()
}

fn node_next<T>(node: *mut Node<T>) -> *mut *mut Node<T> {
    node.cast()
}

#[cfg(test)]
mod tests {
    use core::mem::MaybeUninit;

    use super::Node;

    #[test]
    fn empty() {
        pool!(pub A: [u8; 1]);

        assert!(A::try_alloc().is_none());
    }

    #[test]
    fn sanity() {
        static mut N: MaybeUninit<Node<[u8; 1]>> = MaybeUninit::uninit();

        pool!(pub B: [u8; 1]);
        B::manage(unsafe { &mut N });

        let x = B::try_alloc().unwrap();
        assert!(B::try_alloc().is_none()); // no more memory blocks in the pool
        drop(x); // returns to the pool

        let y = B::try_alloc().unwrap(); // can claim a memory block again
        assert!(B::try_alloc().is_none());
        core::mem::forget(y);
        assert!(B::try_alloc().is_none());
    }
}
