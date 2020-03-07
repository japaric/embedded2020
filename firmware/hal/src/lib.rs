//! Hardware Abstraction Layer

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

pub mod led;
pub mod time;

use cm::{DCB, DWT};
use pac::{CLOCK, P0, RTC0};

#[no_mangle]
unsafe extern "C" fn Reset() {
    // NOTE(borrow_unchecked) interrupts disabled; this runs before user code
    // configure I/O pins
    P0::borrow_unchecked(|p0| {
        // set outputs lows
        p0.OUTSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));

        // set pins as output
        p0.DIRSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
    });
    // seal the above configuration
    P0::seal();

    // use the external 32.768 KHz crystal to drive the low frequency clock
    CLOCK::borrow_unchecked(|clock| {
        // use the external crystal (LFXO) as the low-frequency clock source
        clock.LFCLKSRC.write(|w| w.SRC(1));
        // start the low-frequency clock
        clock.TASKS_LFCLKSTART.write(|w| w.TASKS_LFCLKSTART(1));
        // the clock will starting using the internal RC oscillator (LFRC) and then
        // switch to the LFXO
        while clock.EVENTS_LFCLKSTARTED.read().EVENTS_LFCLKSTARTED() == 0 {
            // wait for the LFXO to become stable
            continue;
        }
    });
    // seal the above configuration
    CLOCK::seal();

    // start the RTC with a counter of 0
    RTC0::borrow_unchecked(|rtc| {
        rtc.TASKS_CLEAR.write(|w| w.TASKS_CLEAR(1));
        rtc.TASKS_START.write(|w| w.TASKS_START(1));
    });
    // seal the above configuration
    RTC0::seal();

    // enable the cycle counter and start it with an initial count of 0
    DCB::borrow_unchecked(|dcb| dcb.DEMCR.rmw(|_, w| w.TRCENA(1)));
    DWT::borrow_unchecked(|dwt| {
        dwt.CYCCNT.write(0);
        dwt.CTRL.rmw(|_, w| w.CYCCNTENA(1));
    });

    // XXX seal DCB & DWT?

    extern "Rust" {
        fn main() -> !;
    }

    main()
}

#[no_mangle]
fn __semidap_timestamp() -> u32 {
    cyccnt() >> 6
}

/// Reads the 32-bit cycle counter
pub fn cyccnt() -> u32 {
    // NOTE(borrow_unchecked) single-instruction read with no side effects
    DWT::borrow_unchecked(|dwt| dwt.CYCCNT.read())
}

#[repr(C)]
union Vector {
    stack_pointer: *const u32,
    handler: unsafe extern "C" fn(),
    reserved: usize,
}

extern "C" {
    static __stack_top__: u32;
    fn NMI();
    fn HardFault();
    fn MemManage();
    fn BusFault();
    fn UsageFault();
    fn SVCall();
    fn DebugMonitor();
    fn PendSV();
    fn SysTick();
    fn DefaultHandler();
}

#[link_section = ".vectors"]
#[no_mangle]
static mut VECTORS: [Vector; 64] = [
    Vector {
        stack_pointer: unsafe { &__stack_top__ as *const u32 },
    },
    Vector { handler: Reset },
    Vector { handler: NMI },
    Vector { handler: HardFault },
    Vector { handler: MemManage },
    Vector { handler: BusFault },
    Vector {
        handler: UsageFault,
    },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: SVCall },
    Vector {
        handler: DebugMonitor,
    },
    Vector { reserved: 0 },
    Vector { handler: PendSV },
    Vector { handler: SysTick },
    // TODO
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector {
        handler: DefaultHandler,
    },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
];
