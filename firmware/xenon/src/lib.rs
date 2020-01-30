#![no_std]

use core::panic::PanicInfo;

pub const RED_LED: u32 = 1 << 13;
pub const GREEN_LED: u32 = 1 << 14;
pub const BLUE_LED: u32 = 1 << 15;
pub const ALL_LEDS: u32 = RED_LED + GREEN_LED + BLUE_LED;

/* NRF52840 MMIO */
const GPIO0_BASE: usize = 0x5000_0000;
pub const GPIO0_OUTSET: *mut u32 = (GPIO0_BASE + 0x508) as *mut u32;
pub const GPIO0_OUTCLR: *mut u32 = (GPIO0_BASE + 0x50c) as *mut u32;
pub const GPIO0_DIRSET: *mut u32 = (GPIO0_BASE + 0x518) as *mut u32;
pub const GPIO0_DIRCLR: *mut u32 = (GPIO0_BASE + 0x51c) as *mut u32;

#[no_mangle]
unsafe extern "C" fn DefaultHandler() -> ! {
    // red LED on
    GPIO0_OUTCLR.write_volatile(RED_LED);
    GPIO0_OUTSET.write_volatile(ALL_LEDS - RED_LED);

    loop {
        continue;
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unsafe { DefaultHandler() }
}
