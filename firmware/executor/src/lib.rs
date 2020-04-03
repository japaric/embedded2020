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
#[inline(always)]
pub fn check<F>(f: F) -> F
where
    F: Future,
{
    f
}

/// Implementation detail
#[doc(hidden)]
#[inline(always)]
pub fn waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        loop {
            continue;
        }
    }

    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe { Waker::from_raw(RawWaker::new(&(), &VTABLE)) }
}
