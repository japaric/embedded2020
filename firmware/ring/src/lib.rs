//! async-aware circular buffer

#![no_std]

use core::{
    cmp, ptr,
    sync::atomic::{self, AtomicU16, Ordering},
};

pub const N: u16 = 256;

pub struct Buffer {
    buffer: *mut u8,
    read: AtomicU16,
    write: AtomicU16,
}

unsafe impl Sync for Buffer {}

impl Buffer {
    /// # Safety
    ///
    /// This is a single-producer single-consumer interrupt-safe circular buffer. All producer
    /// (`write`) operations  need be kept in a single execution context (interrupt priority); the
    /// same requirement applies to the consumer (`read`) operations. Because `Buffer` can be placed
    /// in a `static` variable this requirement needs to be enforced manually
    pub const unsafe fn new(buffer: *const [u8; N as usize]) -> Self {
        Self {
            buffer: buffer as *mut u8,
            read: AtomicU16::new(0),
            write: AtomicU16::new(0),
        }
    }

    pub fn bytes_to_read(&self) -> usize {
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Relaxed);
        write.wrapping_sub(read).into()
    }

    pub fn read(&self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }

        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Relaxed);
        atomic::compiler_fence(Ordering::Acquire);

        let available = write.wrapping_sub(read);
        if available == 0 {
            return 0;
        }

        let cursor = read % N;
        let consumed = cmp::min(buf.len(), available as usize) as u16;
        unsafe {
            ptr::copy_nonoverlapping(
                self.buffer.add(cursor.into()),
                buf.as_mut_ptr(),
                consumed.into(),
            );
        }
        atomic::compiler_fence(Ordering::Release);
        self.read
            .store(read.wrapping_add(consumed), Ordering::Relaxed);
        consumed.into()
    }

    #[cfg(TODO)]
    pub async fn write_all(&self, bytes: &[u8]) {
        todo!()
    }

    pub fn write(&self, bytes: &[u8]) -> usize {
        if bytes.is_empty() {
            return 0;
        }

        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Relaxed);
        atomic::compiler_fence(Ordering::Acquire);

        let available = read.wrapping_add(N).wrapping_sub(write);
        if available == 0 {
            return 0;
        }

        let cursor = write % N;
        let len = cmp::min(bytes.len(), available.into()) as u16;
        unsafe {
            if cursor + len > N {
                // split memcpy
                let pivot = N - cursor;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buffer.add(cursor.into()),
                    pivot.into(),
                );
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(pivot.into()),
                    self.buffer,
                    (len - pivot).into(),
                );
            } else {
                // single memcpy
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buffer.add(cursor.into()),
                    len.into(),
                );
            }
        }
        atomic::compiler_fence(Ordering::Release);
        self.write.store(write.wrapping_add(len), Ordering::Relaxed);
        len.into()
    }
}
