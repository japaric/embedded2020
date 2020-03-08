//! Binary formatting

#![no_main]
#![no_std]

use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    let a = hal::cyccnt();

    semidap::info!("The answer is {}", 0.12345678);
    // ^ this sends the byte sequence:
    // [2, 1, 0, 8, 233, 214, 252, 61]
    //  |  |  |  |  /^^^^^^^^^^^^^^^^
    //  |  |  |  |  +-> `f32.to_le_bytes()`
    //  |  |  |  +----> TAG_F32
    //  |  |  +-------> the footprint (or formatting string) *index*
    //  |  +----------> timestamp in microseconds (LEB128 encoded)
    //  +-------------> TAG_INFO

    let b = hal::cyccnt();

    semidap::info!("That took {} cycles", b - a);

    semidap::exit(0)
}
