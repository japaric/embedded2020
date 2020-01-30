#![feature(asm)]
#![no_std]
#![no_main]

use core::{mem, panic::PanicInfo};

#[cfg(debug_assertions)]
compile_error!("must be compiled in release mode");

const RED: u32 = 1 << 13;
const GREEN: u32 = 1 << 14;
const BLUE: u32 = 1 << 15;
const ALL: u32 = RED | GREEN | BLUE;

/* ARM Cortex-M4 MMIO */
const VTOR: *mut u32 = 0xe000_ed08 as *mut u32;

/* NRF52840 MMIO */
const GPIO0_BASE: usize = 0x5000_0000;
const GPIO0_OUTSET: *mut u32 = (GPIO0_BASE + 0x508) as *mut u32;
const GPIO0_OUTCLR: *mut u32 = (GPIO0_BASE + 0x50c) as *mut u32;
const GPIO0_DIRSET: *mut u32 = (GPIO0_BASE + 0x518) as *mut u32;
const GPIO0_DIRCLR: *mut u32 = (GPIO0_BASE + 0x51c) as *mut u32;

/// Entries in the vector table
const ENTRIES: usize = 64; // NOTE must always be a power of 2
/// Entries used by the NRF42840
const DEVICE_ENTRIES: usize = 37;
const CORTEX_M_ENTRIES: usize = 16;

#[no_mangle]
unsafe extern "C" fn Reset() -> ! {
    // all LEDs off
    GPIO0_OUTSET.write_volatile(ALL);
    // all LED pins as outputs
    GPIO0_DIRSET.write_volatile(ALL);
    // green LED on
    GPIO0_OUTCLR.write_volatile(GREEN);

    let new_vtor: *mut [usize; ENTRIES] = (0x2004_0000 - ENTRIES * mem::size_of::<u32>()) as *mut _;
    let vectors = &*new_vtor;
    let initial_sp = vectors[0];

    // validate the vector table
    // the initial value of the Stack Pointer must be 8-byte aligned
    // all vectors must be odd addresses (thumb bit set to 1)
    if initial_sp % mem::size_of::<u64>() == 0
        && vectors.iter().enumerate().skip(1).all(|(i, vector)| {
            if (7..11).contains(&i) || i == 13 || (CORTEX_M_ENTRIES + DEVICE_ENTRIES..).contains(&i)
            {
                *vector == 0
            } else {
                vector % 2 == 1
            }
        })
    {
        VTOR.write_volatile(new_vtor as u32);

        // turn off the LEDs & make the pins inputs again
        GPIO0_OUTSET.write_volatile(ALL);
        GPIO0_DIRCLR.write_volatile(ALL);

        let reset = vectors[1];

        asm!("
msr MSP, $0
bx $1
" : : "r"(initial_sp) "r"(reset) : : "volatile");
    }

    loop {
        continue;
    }
}

#[no_mangle]
unsafe extern "C" fn DefaultHandler() -> ! {
    // red LED on
    GPIO0_OUTCLR.write_volatile(RED);
    GPIO0_OUTSET.write_volatile(ALL & !RED);

    loop {
        continue;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    extern "C" {
        fn forbidden() -> !;
    }

    unsafe {
        forbidden();
    }
}
