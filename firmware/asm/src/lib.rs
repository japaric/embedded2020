//! Cortex-M assembly
//!
//! `cortex_m::asm` module but with CFI and size information

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

/// Masks interrupts
pub fn disable_irq() {
    extern "C" {
        fn __cpsidi();
    }
    unsafe { __cpsidi() }
}

/// Unmasks interrupts
pub fn enable_irq() {
    extern "C" {
        fn __cpsiei();
    }
    unsafe { __cpsiei() }
}

/// Send EVent
pub fn sev() {
    #[cfg(target_arch = "arm")]
    extern "C" {
        fn __sev();
    }
    #[cfg(target_arch = "arm")]
    unsafe {
        __sev()
    }
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
