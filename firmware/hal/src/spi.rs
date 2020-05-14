//! Serial Peripheral Interface

use core::{
    num::NonZeroU16,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use pac::{p0, SPIM0};

use crate::{p0::Pin, NotSendOrSync};

// TODO hand out up to 3 SPIs
static TAKEN: AtomicBool = AtomicBool::new(false);

/// Host-mode SPI
pub struct Spi {
    _not_send_or_sync: NotSendOrSync,
}

impl Spi {
    /// Turns the given pins into a host-mode SPI
    ///
    /// Frequency will be set to 1 Mbps. SPI will operate in "mode 0"
    pub fn new(sck: Pin, mosi: Pin, miso: Pin) -> Self {
        if TAKEN
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            // pin configuration
            let out_mask = (1 << sck.0) | (1 << mosi.0);
            unsafe { p0::DIRSET::address().write_volatile(out_mask) }
            // SCK & MOSI must be set low (this is default after reset)
            // MISO must be configured as an input (this is the default after reset)

            SPIM0::borrow_unchecked(|spim| {
                // MSB first & mode 0 is the default after reset
                spim.PSEL_MISO.write(|w| w.CONNECT(0).PORT(0).PIN(miso.0));
                spim.PSEL_MOSI.write(|w| w.CONNECT(0).PORT(0).PIN(mosi.0));
                spim.PSEL_SCK.write(|w| w.CONNECT(0).PORT(0).PIN(sck.0));

                // 1 Mbps
                const M1: u32 = 0x1000_0000;
                spim.FREQUENCY.write(|w| w.FREQUENCY(M1));
                spim.ENABLE.write(|w| w.ENABLE(7));
            });

            Spi {
                _not_send_or_sync: NotSendOrSync::new(),
            }
        } else {
            semidap::panic!("`spi` interface has already been claimed");
        }
    }

    /// Reads data from the device by sending it junk data
    pub async fn read(&mut self, buf: &mut [u8]) {
        if let Some(len) = NonZeroU16::new(buf.len() as u16) {
            self.transfer(Transfer::Rx {
                ptr: buf.as_mut_ptr() as u32,
                len,
            })
            .await
        }
    }

    /// Sends data to the device ignoring the data it sends to us
    pub async fn write(&mut self, buf: &[u8]) {
        if let Some(len) = NonZeroU16::new(buf.len() as u16) {
            self.transfer(Transfer::Tx {
                ptr: buf.as_ptr() as u32,
                len,
            })
            .await
        }
    }

    async fn transfer(&mut self, transfer: Transfer) {
        SPIM0::borrow_unchecked(|spim| {
            match transfer {
                Transfer::Rx { ptr, len } => {
                    spim.RXD_MAXCNT.write(|w| w.MAXCNT(len.get()));
                    spim.RXD_PTR.write(|w| w.PTR(ptr));
                    spim.TXD_MAXCNT.write(|w| w.MAXCNT(0));
                }

                Transfer::Tx { ptr, len } => {
                    spim.TXD_MAXCNT.write(|w| w.MAXCNT(len.get()));
                    spim.TXD_PTR.write(|w| w.PTR(ptr));
                    spim.RXD_MAXCNT.write(|w| w.MAXCNT(0));
                }

                Transfer::RxTx {
                    rx_ptr,
                    tx_ptr,
                    rx_len,
                    tx_len,
                } => {
                    spim.RXD_MAXCNT.write(|w| w.MAXCNT(rx_len.get()));
                    spim.RXD_PTR.write(|w| w.PTR(rx_ptr));
                    spim.TXD_MAXCNT.write(|w| w.MAXCNT(tx_len.get()));
                    spim.TXD_PTR.write(|w| w.PTR(tx_ptr));
                }
            }

            crate::dma_start();
            spim.TASKS_START.write(|w| w.TASKS_START(1));
        });

        crate::poll_fn(|| {
            SPIM0::borrow_unchecked(|spim| {
                if spim.EVENTS_END.read().bits() != 0 {
                    crate::dma_end();
                    spim.EVENTS_END.zero();

                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
        })
        .await;
    }
}

enum Transfer {
    Tx {
        ptr: u32,
        len: NonZeroU16,
    },

    Rx {
        ptr: u32,
        len: NonZeroU16,
    },

    #[allow(dead_code)]
    RxTx {
        rx_len: NonZeroU16,
        rx_ptr: u32,
        tx_len: NonZeroU16,
        tx_ptr: u32,
    },
}

/// Chip Select pin
pub struct ChipSelect(u8);

impl ChipSelect {
    /// Configures the given `pin` as a chip-select pin
    pub fn new(pin: Pin) -> Self {
        // configure as output and set high
        unsafe {
            p0::OUTSET::address().write_volatile(1 << pin.0);
            p0::DIRSET::address().write_volatile(1 << pin.0);
        }

        ChipSelect(pin.0)
    }

    /// Selects the SPI device
    pub fn select(&mut self) {
        unsafe { p0::OUTCLR::address().write_volatile(1 << self.0) }
    }

    /// Deselects the SPI device
    pub fn deselect(&mut self) {
        unsafe { p0::OUTSET::address().write_volatile(1 << self.0) }
    }
}
