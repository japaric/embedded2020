use core::{
    marker::PhantomData,
    sync::atomic::{AtomicU8, Ordering},
};

macro_rules! derive {
    ($e:ident) => {
        unsafe impl crate::atomic::Enum for $e {
            unsafe fn from_u8(val: u8) -> Self {
                core::mem::transmute(val)
            }

            fn to_u8(self) -> u8 {
                self as u8
            }
        }
    };
}

pub struct Atomic<E> {
    inner: AtomicU8,
    _marker: PhantomData<E>,
}

/// # Safety
/// - Must be `repr(u8)` C-like enum that covers the range of values `0..=N`, where `N > 0`
pub unsafe trait Enum: Copy {
    unsafe fn from_u8(x: u8) -> Self;
    fn to_u8(self) -> u8;
}

impl<E> Atomic<E> {
    /// Initializes an atomic enum with a value of `0`
    pub const fn new() -> Self {
        Self {
            inner: AtomicU8::new(0),
            _marker: PhantomData,
        }
    }

    pub fn load(&self) -> E
    where
        E: Enum,
    {
        unsafe { E::from_u8(self.inner.load(Ordering::Relaxed)) }
    }

    pub fn store(&self, e: E)
    where
        E: Enum,
    {
        self.inner.store(e.to_u8(), Ordering::Relaxed)
    }
}
