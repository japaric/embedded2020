//! Cortex-M assembly

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

/// Send EVent
pub fn sev() {
    extern "C" {
        fn __sev();
    }
    unsafe { __sev() }
}

/// Wait For Event
pub fn wfe() {
    extern "C" {
        fn __wfe();
    }
    unsafe { __wfe() }
}

/// Wait For Interrupt
pub fn wfi() {
    extern "C" {
        fn __wfi();
    }
    unsafe { __wfi() }
}
