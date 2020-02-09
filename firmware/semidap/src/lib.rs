//! CMSIS-DAP based semihosting

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::{
    cell::{Cell, UnsafeCell},
    cmp,
    convert::Infallible,
    marker::PhantomData,
    mem::MaybeUninit,
    str,
    sync::atomic::{self, AtomicUsize, Ordering},
};

use proc_macro_hack::proc_macro_hack;

/// Logs the formatted string at the `Debug` log level
///
/// A newline will be appended to the end of the format string
#[proc_macro_hack(support_nested)]
pub use semidap_macros::debug;

/// Logs the formatted string at the `Error` log level
///
/// A newline will be appended to the end of the format string
#[proc_macro_hack(support_nested)]
pub use semidap_macros::error;

/// Logs the formatted string at the `Info` log level
///
/// A newline will be appended to the end of the format string
#[proc_macro_hack(support_nested)]
pub use semidap_macros::info;

/// Logs the formatted string at the `Trace` log level
///
/// A newline will be appended to the end of the format string
#[proc_macro_hack(support_nested)]
pub use semidap_macros::trace;

/// Logs the formatted string at the `Warn` log level
///
/// A newline will be appended to the end of the format string
#[proc_macro_hack(support_nested)]
pub use semidap_macros::warn;

/// Prints the formatted string to the host console
#[proc_macro_hack(support_nested)]
pub use semidap_macros::print;

/// Prints the formatted string to the host console
///
/// A newline will be appended to the end of the format string
#[proc_macro_hack(support_nested)]
pub use semidap_macros::println;

fn in_thread_mode() -> bool {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;

    unsafe { SCB_ICSR.read_volatile() as u8 == 0 }
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
pub struct Stdout {
    _not_send_or_sync: PhantomData<*mut ()>,
}

/// Implementation detail
/// # Safety
/// None of `Stdout` methods are re-entrable
#[doc(hidden)]
pub fn stdout() -> Option<Stdout> {
    if in_thread_mode() {
        Some(Stdout {
            _not_send_or_sync: PhantomData,
        })
    } else {
        None
    }
}

/// Implementation detail
#[doc(hidden)]
#[repr(C)]
pub struct Cursor {
    // TODO(?) use `AtomicU16` instead of `AtomicUsize` then both cursors can be
    // read in a single instruction
    // NOTE this field can be modified by the debugger
    read: AtomicUsize,
    write: AtomicUsize,
}

impl Cursor {
    const fn new() -> Self {
        Self {
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
        }
    }
}

const CAPACITY: usize = 256;

// TODO support logging in interrupt context. Place all the cursors in an array
// so the host can read them in a single DAP transaction (`DAP_TransferBlock`)
// Host visible
#[link_section = ".uninit.SEMIDAP_BUFFER"]
#[no_mangle]
static mut SEMIDAP_BUFFER: UnsafeCell<MaybeUninit<[u8; CAPACITY]>> =
    UnsafeCell::new(MaybeUninit::uninit());

#[no_mangle]
static SEMIDAP_CURSOR: Cursor = Cursor::new();

// Only visible to the target
#[link_section = ".uninit.BUFFER"]
static mut BUFFER: UnsafeCell<MaybeUninit<[u8; CAPACITY]>> =
    UnsafeCell::new(MaybeUninit::uninit());

static mut CURSOR: Cell<u8> = Cell::new(0);

impl Stdout {
    #[cold]
    #[doc(hidden)]
    pub fn flush(&mut self) {
        unsafe {
            let dst = SEMIDAP_BUFFER.get() as *mut u8;
            let mut src = BUFFER.get() as *const u8;

            let mut left = usize::from(CURSOR.get());
            let mut write = SEMIDAP_CURSOR.write.load(Ordering::Relaxed);
            while left != 0 {
                let read = SEMIDAP_CURSOR.read.load(Ordering::Relaxed);
                atomic::compiler_fence(Ordering::Acquire);

                let free = read.wrapping_add(CAPACITY).wrapping_sub(write);
                if free == 0 {
                    // busy wait
                    continue;
                }

                let step = cmp::min(left, free);
                let cursor = write % CAPACITY;

                if cursor + step > CAPACITY {
                    // split memcpy
                    let m = cursor.wrapping_add(step).wrapping_sub(CAPACITY);

                    // write cursor to end
                    memcpy(src, dst.add(cursor), step);

                    // wrap-around to the beginning
                    memcpy(src.add(m), dst, step - m);
                } else {
                    // single memcpy
                    memcpy(src, dst.add(cursor), step);
                }

                left -= step;
                src = src.add(step);
                write = write.wrapping_add(step);
                atomic::compiler_fence(Ordering::Release);
                SEMIDAP_CURSOR.write.store(write, Ordering::Relaxed);
            }

            CURSOR.set(0);
        }
    }

    #[doc(hidden)]
    pub fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes())
    }

    // NOTE currently limited to 256 strings but we could use LEB128 encoding
    // here
    #[doc(hidden)]
    pub fn write_sym(&mut self, sym: *const u8) {
        self.write(&[consts::UTF8_SYMTAB_STRING, sym as u8])
    }

    #[doc(hidden)]
    pub fn write_timestamp(&mut self) {
        extern "Rust" {
            // NOTE currently this is always a 24-bit value
            fn semidap_timestamp() -> u32;
        }

        let ts = unsafe { semidap_timestamp() };
        // little endian encoding
        self.write(&[
            consts::UTF8_TIMESTAMP,
            ts as u8,
            (ts >> 8) as u8,
            (ts >> 16) as u8,
        ])
    }

    #[doc(hidden)]
    fn write(&mut self, bytes: &[u8]) {
        unsafe {
            let mut cursor = CURSOR.get().into();
            let len = bytes.len();

            if cursor + len > CAPACITY {
                self.flush();
                cursor = 0;
            }

            memcpy(bytes.as_ptr(), (BUFFER.get() as *mut u8).add(cursor), len);
            CURSOR.set((cursor + len) as u8)
        }
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

impl ufmt_write::uWrite for Stdout {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        self.write_str(s);
        Ok(())
    }
}

/// Checks the condition and aborts the program if it evaluates to `false`
// TODO turn into a procedural macro to merge the strings at compile time
#[macro_export]
macro_rules! assert {
    ($e:expr) => {
        $crate::assert!($e, "assertion failed: {}", stringify!($e))
    };

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
