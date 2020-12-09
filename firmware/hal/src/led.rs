//! LEDs

use pac::p0;

// MDK
// pub(crate) const RED: u32 = 1 << 23;
// pub(crate) const GREEN: u32 = 1 << 22;
// pub(crate) const BLUE: u32 = 1 << 24;

// Dongle
pub(crate) const RED: u32 = 1 << 8;
pub(crate) const GREEN: u32 = 1 << 6;
pub(crate) const BLUE: u32 = 1 << 12;

// DK
// pub(crate) const RED: u32 = 1 << 13;
// pub(crate) const GREEN: u32 = 1 << 14;
// pub(crate) const BLUE: u32 = 1 << 15;

/// Red LED
pub struct Red;

impl Red {
    /// Turns the LED off
    pub fn off(&self) {
        unsafe { p0::OUTSET::address().write_volatile(RED) }
    }

    /// Turns the LED on
    pub fn on(&self) {
        unsafe { p0::OUTCLR::address().write_volatile(RED) }
    }
}

/// Green LED
pub struct Green;

impl Green {
    /// Turns the LED off
    pub fn off(&self) {
        unsafe { p0::OUTSET::address().write_volatile(GREEN) }
    }

    /// Turns the LED on
    pub fn on(&self) {
        unsafe { p0::OUTCLR::address().write_volatile(GREEN) }
    }
}

/// Blue LED
pub struct Blue;

impl Blue {
    /// Turns the LED off
    pub fn off(&self) {
        unsafe { p0::OUTSET::address().write_volatile(BLUE) }
    }

    /// Turns the LED on
    pub fn on(&self) {
        unsafe { p0::OUTCLR::address().write_volatile(BLUE) }
    }
}
