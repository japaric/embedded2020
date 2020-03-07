//! CMSIS-DAP based semihosting

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
};

#[doc(hidden)]
pub use binfmt::{binWrite, binwriteln, binwriteln_, Level};

/// Prints the formatted string to the host console
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! println {
    ($($tt:tt)+) => {
        match $crate::stdout() {
            ref mut __stdout__ => {
                $crate::binwriteln!(&mut __stdout__, $($tt)+)
            }
        }
    }
}

/// Logs the formatted string at the `Debug` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! debug {
    ($($tt:tt)+) => {
        match () {
            #[cfg(debug_assertions)]
            () => {
                match $crate::stdout() {
                    ref mut __stdout__ => {
                        $crate::log(__stdout__, $crate::Level::Debug);
                        $crate::binwriteln_!(__stdout__, $($tt)+)
                    }
                }
            }
            #[cfg(not(debug_assertions))]
            () => {}
        }
    }
}

/// Logs the formatted string at the `Error` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! error {
    ($($tt:tt)+) => {
        match $crate::stdout() {
            ref mut __stdout__ => {
                $crate::log(__stdout__, $crate::Level::Error);
                $crate::binwriteln_!(__stdout__, $($tt)+)
            }
        }
    }
}

/// Logs the formatted string at the `Info` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! info {
    ($($tt:tt)+) => {
        match $crate::stdout() {
            ref mut __stdout__ => {
                $crate::log(__stdout__, $crate::Level::Info);
                $crate::binwriteln_!(__stdout__, $($tt)+)
            }
        }
    }
}

/// Logs the formatted string at the `Trace` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! trace {
    ($($tt:tt)+) => {
        match () {
            #[cfg(debug_assertions)]
            () => {
                match $crate::stdout() {
                    ref mut __stdout__ => {
                        $crate::log(__stdout__, $crate::Level::Trace);
                        $crate::binwriteln_!(__stdout__, $($tt)+)
                    }
                }
            }
            #[cfg(not(debug_assertions))]
            () => {}
        }
    }
}

/// Logs the formatted string at the `Warn` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! warn {
    ($($tt:tt)+) => {
        match $crate::stdout() {
            ref mut __stdout__ => {
                $crate::log(__stdout__, $crate::Level::Warn);
                $crate::binwriteln_!(__stdout__, $($tt)+)
            }
        }
    }
}

fn in_thread_mode() -> bool {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;

    unsafe { SCB_ICSR.read_volatile() as u8 == 0 }
}

#[doc(hidden)]
pub fn log(stdout: &mut impl binWrite, level: Level) {
    extern "Rust" {
        fn __semidap_timestamp() -> u32;
    }
    let ts = unsafe { __semidap_timestamp() };
    stdout.write_byte(level as u8);
    stdout.leb128_write(ts);
}

/// Aborts the `semidap` process running on the host
#[inline(always)]
pub fn abort() -> ! {
    extern "C" {
        fn __abort() -> !;
    }
    unsafe { __abort() }
}

/// Exits the `semidap` process running on the host with the specified exit code
#[inline(always)]
pub fn exit(code: i32) -> ! {
    extern "C" {
        fn __exit(r0: i32) -> !;
    }
    unsafe { __exit(code) }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct Channel {
    bufferp: *mut u8,
    // the `read` pointer is kept in host memory
    write: &'static Cell<u16>,
}

/// Implementation detail
/// # Safety
/// None of `Channel` methods are re-entrant safe
#[doc(hidden)]
pub fn stdout() -> Channel {
    if in_thread_mode() {
        unsafe { CHANNELS[0] }
    } else {
        // TODO one channel per priority level
        unsafe { CHANNELS[1] }
    }
}

// TODO(?) change this to the *usable* size of one HID packet
const HID_PACKET_SIZE: u8 = 64;
const CAPACITY: u16 = 8 * HID_PACKET_SIZE as u16;

#[no_mangle]
static mut SEMIDAP_CURSOR: [Cell<u16>; 2] = [Cell::new(0), Cell::new(0)];
#[link_section = ".uninit.SEMIDAP_BUFFER"]
#[no_mangle]
static mut SEMIDAP_BUFFER: [UnsafeCell<
    MaybeUninit<[u8; 2 * CAPACITY as usize]>,
>; 2] = [
    UnsafeCell::new(MaybeUninit::uninit()),
    UnsafeCell::new(MaybeUninit::uninit()),
];

static mut CHANNELS: [Channel; 2] = unsafe {
    [
        Channel {
            write: &SEMIDAP_CURSOR[0],
            bufferp: &SEMIDAP_BUFFER[0] as *const _ as *mut u8,
        },
        Channel {
            write: &SEMIDAP_CURSOR[1],
            bufferp: &SEMIDAP_BUFFER[1] as *const _ as *mut u8,
        },
    ]
};

impl Channel {
    fn push(&self, byte: u8) {
        let write = self.write.get();
        let cursor = write % CAPACITY;
        unsafe { self.bufferp.add(cursor.into()).write(byte) }
        self.write.set(write.wrapping_add(1));
    }

    fn extend_from_slice(&self, bytes: &[u8]) {
        // NOTE we assume that `bytes.len` is less than `u16::max_value` which
        // is very likely to be the case as logs are compressed
        let len = bytes.len() as u16;
        let write = self.write.get();
        let cursor = write % CAPACITY;

        // NOTE it might be worth the do writes in `HID_PACKET_SIZE` chunks to
        // improve the chances of the host advancing its `read` pointer during
        // the execution of this method. OTOH, it's very unlikely that
        // `bytes.len()` will be greater than `HID_PACKET_SIZE`
        if cursor + len > CAPACITY {
            // split memcpy
            // NOTE here we assume that `bytes.len()` is less than `CAPACITY`.
            // When that's not the case the second `memcpy` could result in an
            // out of bounds write. It's very unlikely that `bytes.len()` will
            // ever be greater than `CAPACITY` because logs are compressed
            let pivot = cursor.wrapping_add(len).wrapping_sub(CAPACITY);
            unsafe {
                memcpy(
                    bytes.as_ptr(),
                    self.bufferp.add(cursor.into()),
                    pivot.into(),
                );
                memcpy(
                    bytes.as_ptr().add(pivot.into()),
                    self.bufferp,
                    (len - pivot).into(),
                );
            }
        } else {
            // single memcpy
            unsafe {
                memcpy(
                    bytes.as_ptr(),
                    self.bufferp.add(cursor.into()),
                    len.into(),
                )
            }
        }

        // NOTE we want the `write` cursor to always be updated after `bufferp`.
        // we may need a compiler barrier here
        self.write.set(write.wrapping_add(len));
    }
}

impl binfmt::binWrite for Channel {
    fn write_byte(&mut self, byte: u8) {
        self.push(byte)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes)
    }
}

// opt-level = 3
#[cfg(not(debug_assertions))]
unsafe fn memcpy(src: *const u8, dst: *mut u8, len: usize) {
    // lowers to `__aebi_memcpy`
    core::ptr::copy_nonoverlapping(src, dst, len);
}

// opt-level = 'z'
#[cfg(debug_assertions)]
unsafe fn memcpy(mut src: *const u8, mut dst: *mut u8, len: usize) {
    // lowers to a tight loop (less instructions than `__aeabi_memcpy`)
    for _ in 0..len {
        dst.write_volatile(src.read_volatile());
        dst = dst.add(1);
        src = src.add(1);
    }
}

/// Checks the condition and aborts the program if it evaluates to `false`
#[macro_export]
macro_rules! assert {
    ($e:expr, $($tt:tt)*) => {
        if !$e {
            $crate::panic!($($tt)*)
        }
    };
}

/// Prints an `Error` message and aborts the program
#[macro_export]
macro_rules! panic {
    () => {
        $crate::abort()
    };

    ($($tt:tt)*) => {{
        $crate::error!($($tt)*);
        $crate::abort()
    }}
}
