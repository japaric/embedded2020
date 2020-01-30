#![no_main]
#![no_std]

#[no_mangle]
unsafe extern "C" fn Reset() -> ! {
    // all LEDs off
    xenon::GPIO0_OUTSET.write_volatile(xenon::ALL_LEDS);
    // all LED pins as outputs
    xenon::GPIO0_DIRSET.write_volatile(xenon::ALL_LEDS);
    // blue LED on
    xenon::GPIO0_OUTCLR.write_volatile(xenon::BLUE_LED);

    loop {
        continue;
    }
}
