//! Port 0

use core::sync::atomic::{AtomicBool, Ordering};

/// Port 0
pub struct P0 {
    // pub pin0: Pin, // not routed (?)
    // pub pin1: Pin, // not routed (?)
    /// P0.2
    pub pin2: Pin,
    /// P0.3
    pub pin3: Pin,
    /// P0.4
    pub pin4: Pin,
    /// P0.5
    pub pin5: Pin,
    /// P0.6
    pub pin6: Pin,
    /// P0.7
    pub pin7: Pin,
    /// P0.8
    pub pin8: Pin,
    // pub pin9: Pin, // NFC1
    // pub pin10: Pin, // NRF2
    /// P0.11
    pub pin11: Pin,
    /// P0.12
    pub pin12: Pin,
    /// P0.13
    pub pin13: Pin,
    /// P0.14
    pub pin14: Pin,
    /// P0.15
    pub pin15: Pin,
    /// P0.16
    pub pin16: Pin,
    /// P0.17
    pub pin17: Pin,
    // pub pin18: Pin, // not routed (?)
    // pub pin19: Pin, // RXD
    // pub pin20: Pin, // TXD
    /// P0.21
    pub pin21: Pin,
    // pub pin22: Pin, // green LED
    // pub pin23: Pin, // red LED
    // pub pin24: Pin, // blue LED
    /// P0.25
    pub pin25: Pin,
    /// P0.26
    pub pin26: Pin,
    /// P0.27
    pub pin27: Pin,
    /// P0.28
    pub pin28: Pin,
    /// P0.29
    pub pin29: Pin,
    /// P0.30
    pub pin30: Pin,
    /// P0.31
    pub pin31: Pin,
}

/// Output level
#[derive(Clone, Copy, PartialEq)]
pub enum Level {
    /// Low level (0)
    Low,
    /// High level (1)
    High,
}

/// Output pin
pub struct Output(pub(crate) u8);

impl Output {
    /// Changes the output `level` of the pin
    pub fn set(&mut self, level: Level) {
        let mask = 1 << self.0;

        unsafe {
            match level {
                Level::Low => pac::p0::OUTCLR::address().write_volatile(mask),
                Level::High => pac::p0::OUTSET::address().write_volatile(mask),
            }
        }
    }

    /// Sets the pin high
    pub fn set_high(&mut self) {
        self.set(Level::High)
    }

    /// Sets the pin low
    pub fn set_low(&mut self) {
        self.set(Level::Low)
    }
}

/// P0 pin
pub struct Pin(pub(crate) u8);

impl Pin {
    /// Configures the pin as an input pin
    #[cfg(TODO)]
    pub fn into_input(self, pullup: Level) -> Input {
        unsafe {
            let (pu, sense) = match pullup {
                Level::Low => (1, 2),
                Level::High => (3, 3),
            };

            let mut w = pac::p0::pin_cnf0::W::zero();
            w.INPUT(1).PULL(pu).SENSE(sense);
            pac::p0::PIN_CNF0::address()
                .offset(self.0.into())
                .write_volatile(w.into());
        }

        Input(self.0)
    }

    /// Configures the pin as an output pin
    pub fn into_output(self, level: Level) -> Output {
        unsafe {
            if level == Level::High {
                pac::p0::OUTSET::address().write_volatile(1 << self.0);
                pac::p0::DIRSET::address().write_volatile(1 << self.0);
            }

            Output(self.0)
        }
    }
}

static TAKEN: AtomicBool = AtomicBool::new(false);

/// Returns all P0 pins
pub fn claim() -> P0 {
    if TAKEN
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        P0 {
            pin2: Pin(2),
            pin3: Pin(3),
            pin4: Pin(4),
            pin5: Pin(5),
            pin6: Pin(6),
            pin7: Pin(7),
            pin8: Pin(8),
            pin11: Pin(11),
            pin12: Pin(12),
            pin13: Pin(13),
            pin14: Pin(14),
            pin15: Pin(15),
            pin16: Pin(16),
            pin17: Pin(17),
            pin21: Pin(21),
            pin25: Pin(25),
            pin26: Pin(26),
            pin27: Pin(27),
            pin28: Pin(28),
            pin29: Pin(29),
            pin30: Pin(30),
            pin31: Pin(31),
        }
    } else {
        semidap::panic!("port `p0` has already been claimed")
    }
}
