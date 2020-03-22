//! ARM Cortex-M executor
//!
//! # Features
//!
//! - No heap allocations
//! - No trait objects
//! - Tasks do NOT need to satisfy the `: 'static` bound

#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

use core::{
    future::Future,
    sync::atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering},
    task::{RawWaker, RawWakerVTable, Waker},
};

use proc_macro_hack::proc_macro_hack;

/// Implementation detail
#[doc(hidden)]
pub use asm::wfe;

/// Runs the given tasks concurrently
///
/// This macro is divergent (`-> !`); the tasks should also be divergent
#[proc_macro_hack(support_nested)]
pub use executor_macros::run;

/// Implementation detail
#[doc(hidden)]
#[repr(align(2))]
#[repr(C)]
pub struct Flags2 {
    inner: [AtomicBool; 2],
}

impl Flags2 {
    /// Implementation detail
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self {
            inner: [AtomicBool::new(true), AtomicBool::new(true)],
        }
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn next(&self) -> Option<(u8, &AtomicBool)> {
        let mask = self.mask();
        if mask == 0 {
            None
        } else {
            let i = if mask & 1 != 0 { 0 } else { 1 };

            Some((i, unsafe { self.inner.get_unchecked(usize::from(i)) }))
        }
    }

    fn mask(&self) -> u16 {
        unsafe { (*(&self.inner as *const _ as *const AtomicU16)).load(Ordering::Relaxed) }
    }
}

/// Implementation detail
#[doc(hidden)]
#[repr(align(4))]
#[repr(C)]
pub struct Flags4 {
    inner: [AtomicBool; 4],
}

impl Flags4 {
    /// Implementation detail
    #[doc(hidden)]
    pub const fn new() -> Self {
        Self {
            inner: [
                AtomicBool::new(true),
                AtomicBool::new(true),
                AtomicBool::new(true),
                AtomicBool::new(true),
            ],
        }
    }

    /// Implementation detail
    #[doc(hidden)]
    pub fn next(&self) -> Option<(u8, &AtomicBool)> {
        let mask = self.mask();
        if mask == 0 {
            None
        } else {
            let i = if mask & 1 != 0 {
                0
            } else if mask & 0x100 != 0 {
                1
            } else if mask & 0x1_0000 != 0 {
                2
            } else {
                3
            };

            Some((i, unsafe { self.inner.get_unchecked(usize::from(i)) }))
        }
    }

    fn mask(&self) -> u32 {
        unsafe { (*(&self.inner as *const _ as *const AtomicU32)).load(Ordering::Relaxed) }
    }
}

/// Implementation detail
#[doc(hidden)]
#[inline(always)]
pub fn check<F>(f: F) -> F
where
    F: Future,
{
    f
}

/// Implementation detail
/// # Safety
/// `flag` must point to a value that will never be deallocated
#[doc(hidden)]
pub fn waker(ready: &'static AtomicBool) -> Waker {
    unsafe fn clone(ready: *const ()) -> RawWaker {
        RawWaker::new(ready, &VTABLE)
    }

    unsafe fn wake(ready: *const ()) {
        wake_by_ref(ready)
    }

    unsafe fn wake_by_ref(ready: *const ()) {
        // set the event register so they next `wfe` becomes a NOP
        // this also causes the "thread" handler to wake up from a `wfe`
        asm::sev();
        (*(ready as *const AtomicBool)).store(true, Ordering::Relaxed)
    }

    unsafe fn drop(_ready: *const ()) {
        // nothing to do: `flag` will never be deallocated
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe {
        Waker::from_raw(RawWaker::new(
            ready as *const AtomicBool as *const (),
            &VTABLE,
        ))
    }
}
