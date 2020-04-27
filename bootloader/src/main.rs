#![deny(warnings)]
#![no_main]
#![no_std]

use core::mem;

use cm::SCB;
use pac::P0;
use panic_never as _; // this bootloader contains no panicking branches

#[cfg(debug_assertions)]
compile_error!("must be compiled in release mode");

/// End of the RAM memory region
const RAM_END: usize = 0x2004_0000;

/// Entries in the vector table
const ENTRIES: usize = 64; // NOTE must always be a power of 2

#[no_mangle]
unsafe extern "C" fn Reset() -> ! {
    P0::borrow_unchecked(|p0| {
        // all LEDs off
        p0.OUTSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
        // all LED pins as outputs
        p0.DIRSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
        // green LED on
        p0.OUTCLR.write(|w| w.PIN14(1));
    });

    let new_vtor: *mut [usize; ENTRIES] = (RAM_END - ENTRIES * mem::size_of::<u32>()) as *mut _;
    let vectors = &*new_vtor;
    let initial_sp = vectors[0];

    // validate the vector table
    // the initial value of the Stack Pointer must be 8-byte aligned
    // all vectors must be odd addresses (thumb bit set to 1)
    let check_vector = |(i, vector): (usize, &usize)| {
        if (7..11).contains(&i)
            || i == 13
            || (46..48).contains(&i)
            || (59..61).contains(&i)
            || i == 62
        {
            *vector == 0
        } else {
            vector % 2 == 1
        }
    };
    if initial_sp % mem::size_of::<u64>() == 0
        && vectors.iter().enumerate().skip(1).all(check_vector)
    {
        SCB::borrow_unchecked(|scb| scb.VTOR.write(|w| w.TBLOFF(new_vtor as u32 >> 7)));

        P0::borrow_unchecked(|p0| {
            // all LEDs off
            p0.OUTSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
            // make pin inputs
            p0.DIRCLR.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
        });

        let reset = vectors[1];

        extern "C" {
            fn __jump(initial_sp: usize, reset: usize) -> !;
        }

        __jump(initial_sp, reset)
    } else {
        loop {
            continue;
        }
    }
}

#[no_mangle]
unsafe extern "C" fn DefaultHandler() -> ! {
    P0::borrow_unchecked(|p0| {
        // all LEDs off
        p0.OUTCLR.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
        // red LED on
        p0.OUTSET.write(|w| w.PIN13(1));
    });

    loop {
        continue;
    }
}
