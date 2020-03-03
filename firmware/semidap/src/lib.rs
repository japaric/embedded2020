//! CMSIS-DAP based semihosting

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::{
    cell::{Cell, UnsafeCell},
    cmp,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{self, AtomicUsize, Ordering},
};

#[doc(hidden)]
pub use binfmt::{binWrite, binwriteln, Level};

/// Prints the formatted string to the host console
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! println {
    ($($tt:tt)+) => {
        if let Some(mut __stdout__) = $crate::stdout() {
            $crate::binwriteln!(&mut __stdout__, $($tt)+)
        }
    }
}

/// Logs the formatted string at the `Debug` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! debug {
    ($($tt:tt)+) => {
        if cfg!(debug_assertions) {
            if let Some(mut __stdout__) = $crate::stdout() {
                $crate::log(&mut __stdout__, $crate::Level::Debug);
                $crate::binwriteln!(&mut __stdout__, $($tt)+)
            }
        }
    }
}

/// Logs the formatted string at the `Error` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! error {
    ($($tt:tt)+) => {
        if let Some(mut __stdout__) = $crate::stdout() {
            $crate::log(&mut __stdout__, $crate::Level::Error);
            $crate::binwriteln!(&mut __stdout__, $($tt)+)
        }
    }
}

/// Logs the formatted string at the `Info` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! info {
    ($($tt:tt)+) => {
        if let Some(mut __stdout__) = $crate::stdout() {
            $crate::log(&mut __stdout__, $crate::Level::Info);
            $crate::binwriteln!(&mut __stdout__, $($tt)+)
        }
    }
}

/// Logs the formatted string at the `Trace` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! trace {
    ($($tt:tt)+) => {
        if cfg!(debug_assertions) {
            if let Some(mut __stdout__) = $crate::stdout() {
                $crate::log(&mut __stdout__, $crate::Level::Trace);
                $crate::binwriteln!(&mut __stdout__, $($tt)+)
            }
        }
    }
}

/// Logs the formatted string at the `Warn` log level
///
/// A newline will be appended to the end of the format string
#[macro_export]
macro_rules! warn {
    ($($tt:tt)+) => {
        if let Some(mut __stdout__) = $crate::stdout() {
            $crate::log(&mut __stdout__, $crate::Level::Warn);
            $crate::binwriteln!(&mut __stdout__, $($tt)+)
        }
    }
}

/// Flushes the local buffer
// TODO expose the local `BUFFER` to the host so it can drain it in `abort` and
// stack overflow scenarios; then make this private again
pub fn flush() {
    if let Some(mut stdout) = stdout() {
        stdout.flush();
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
    stdout.log(level, ts);
}

/// Aborts the `semidap` process running on the host
#[inline(always)]
pub fn abort() -> ! {
    extern "C" {
        fn __abort() -> !;
    }
    flush();
    unsafe { __abort() }
}

/// Exits the `semidap` process running on the host with the specified exit code
#[inline(always)]
pub fn exit(code: i32) -> ! {
    extern "C" {
        fn __exit(r0: i32) -> !;
    }
    flush();
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

const SHARED_CAPACITY: usize = 4 * LOCAL_CAPACITY; // holds ~4 HID packets
const LOCAL_CAPACITY: usize = 64; // size of 1 HID packet

// TODO support logging in interrupt context. Place all the cursors in an array
// so the host can read them in a single DAP transaction (`DAP_TransferBlock`)
// Host visible
#[link_section = ".uninit.SEMIDAP_BUFFER"]
#[no_mangle]
static mut SEMIDAP_BUFFER: UnsafeCell<MaybeUninit<[u8; SHARED_CAPACITY]>> =
    UnsafeCell::new(MaybeUninit::uninit());

#[no_mangle]
static SEMIDAP_CURSOR: Cursor = Cursor::new();

// XXX does this actually speed up things?
// Only visible to the target
#[link_section = ".uninit.BUFFER"]
static mut BUFFER: UnsafeCell<MaybeUninit<[u8; LOCAL_CAPACITY]>> =
    UnsafeCell::new(MaybeUninit::uninit());

static mut CURSOR: Cell<u8> = Cell::new(0);

impl Stdout {
    #[cold]
    fn flush(&mut self) {
        unsafe {
            let dst = SEMIDAP_BUFFER.get() as *mut u8;
            let mut src = BUFFER.get() as *const u8;

            let mut left = usize::from(CURSOR.get());
            let mut write = SEMIDAP_CURSOR.write.load(Ordering::Relaxed);
            while left != 0 {
                let read = SEMIDAP_CURSOR.read.load(Ordering::Relaxed);
                atomic::compiler_fence(Ordering::Acquire);

                let free =
                    read.wrapping_add(SHARED_CAPACITY).wrapping_sub(write);
                if free == 0 {
                    // busy wait
                    continue;
                }

                let step = cmp::min(left, free);
                let cursor = write % SHARED_CAPACITY;

                if cursor + step > SHARED_CAPACITY {
                    // split memcpy
                    let m =
                        cursor.wrapping_add(step).wrapping_sub(SHARED_CAPACITY);

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

    fn write(&mut self, bytes: &[u8]) {
        unsafe {
            let mut len = bytes.len();
            let mut p = bytes.as_ptr();

            while len != 0 {
                let cursor = CURSOR.get();
                let n = cmp::min(len, LOCAL_CAPACITY - cursor as usize);
                memcpy(p, (BUFFER.get() as *mut u8).add(cursor.into()), n);
                CURSOR.set(cursor + n as u8);
                len -= n;
                p = p.add(n);
                if cursor as usize + len > LOCAL_CAPACITY {
                    self.flush();
                }
            }
        }
    }
}

impl binfmt::binWrite for Stdout {
    fn write(&mut self, bytes: &[u8]) {
        self.write(bytes)
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
