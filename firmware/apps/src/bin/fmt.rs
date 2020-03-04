//! Binary formatting

#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    let a = hal::cyccnt();

    semidap::info!("The answer is {}", 0.12345678);
    // ^ this sends the byte sequence:
    // [2, 0, 5, 0, 8, 233, 214, 252, 61]
    //  |  |  |  |  |  ^^^^^^^^^^^^^^^^^ the `f32` value as Little Endian bytes
    //  |  |  |  |  +-> TAG_F32
    //  |  |  |  +----> the footprint (or formatting string) *index*
    //  |  |  +-------> TAG_FOOTPRINT
    //  |  +----------> timestamp in 32,768 Hz ticks (LEB128 encoded)
    //  +-------------> TAG_INFO

    let b = hal::cyccnt();

    semidap::info!("That took {} cycles", b - a);

    semidap::exit(0)
}
