//! Hardware Abstraction Layer

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

mod errata;
pub mod led;
mod reset;
pub mod time;
mod usbd;

use cm::{DWT, NVIC};
use pac::FICR;

/// Reads the 32-bit cycle counter
pub fn cyccnt() -> u32 {
    // NOTE(borrow_unchecked) single-instruction read with no side effects
    DWT::borrow_unchecked(|dwt| dwt.CYCCNT.read())
}

/// Returns the device identifier
pub fn deviceid() -> u64 {
    // NOTE(borrow_unchecked) read-only registers
    FICR::borrow_unchecked(|ficr| {
        u64::from(ficr.DEVICEID0.read().bits()) | u64::from(ficr.DEVICEID1.read().bits()) << 32
    })
}

#[allow(dead_code)]
unsafe fn unmask0(interrupts: &[Interrupt0]) {
    let mut val = 0;
    for interrupt in interrupts.iter().cloned() {
        val |= 1 << interrupt as u8;
    }

    if val != 0 {
        // NOTE(borrow_unchecked) single-instruction write
        NVIC::borrow_unchecked(|nvic| nvic.ISER0.write(val));
    }
}

unsafe fn unmask1(interrupts: &[Interrupt1]) {
    let mut val = 0;
    for interrupt in interrupts.iter().cloned() {
        val |= 1 << (interrupt as u8 - 32);
    }

    if val != 0 {
        // NOTE(borrow_unchecked) single-instruction write
        NVIC::borrow_unchecked(|nvic| nvic.ISER1.write(val));
    }
}

/// Interrupts 0..32
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Interrupt0 {
    POWER_CLOCK = 0,
    RADIO = 1,
    UARTE0_UART0 = 2,
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 = 3,
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 = 4,
    NFCT = 5,
    GPIOTE = 6,
    SAADC = 7,
    TIMER0 = 8,
    TIMER1 = 9,
    TIMER2 = 10,
    RTC0 = 11,
    TEMP = 12,
    RNG = 13,
    ECB = 14,
    CCM_AAR = 15,
    WDT = 16,
    RTC1 = 17,
    QDEC = 18,
    COMP_LPCOMP = 19,
    SWI0_EGU0 = 20,
    SWI1_EGU1 = 21,
    SWI2_EGU2 = 22,
    SWI3_EGU3 = 23,
    SWI4_EGU4 = 24,
    SWI5_EGU5 = 25,
    TIMER3 = 26,
    TIMER4 = 27,
    PWM0 = 28,
    PDM = 29,
}

/// Interrupts 32..
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Interrupt1 {
    MWU = 32,
    PWM1 = 33,
    PWM2 = 34,
    SPIM2_SPIS2_SPI2 = 35,
    RTC2 = 36,
    I2S = 37,
    FPU = 38,
    USBD = 39,
    UARTE1 = 40,
    QSPI = 41,
    CRYPTOCELL = 42,
    PWM3 = 45,
    SPIM3 = 47,
}
