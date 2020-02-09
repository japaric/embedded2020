//! LEDs
use pac::P0;

/// Red LED
pub struct Red;

impl Red {
    /// Turns the LED off
    pub fn off(&self) {
        P0::borrow_unchecked(|p0| p0.OUTSET.write(|w| w.PIN13(1)))
    }

    /// Turns the LED on
    pub fn on(&self) {
        P0::borrow_unchecked(|p0| p0.OUTCLR.write(|w| w.PIN13(1)))
    }
}

/// Green LED
pub struct Green;

impl Green {
    /// Turns the LED off
    pub fn off(&self) {
        P0::borrow_unchecked(|p0| p0.OUTSET.write(|w| w.PIN14(1)))
    }

    /// Turns the LED on
    pub fn on(&self) {
        P0::borrow_unchecked(|p0| p0.OUTCLR.write(|w| w.PIN14(1)))
    }
}

/// Blue LED
pub struct Blue;

impl Blue {
    /// Turns the LED off
    pub fn off(&self) {
        P0::borrow_unchecked(|p0| p0.OUTSET.write(|w| w.PIN15(1)))
    }

    /// Turns the LED on
    pub fn on(&self) {
        P0::borrow_unchecked(|p0| p0.OUTCLR.write(|w| w.PIN15(1)))
    }
}
