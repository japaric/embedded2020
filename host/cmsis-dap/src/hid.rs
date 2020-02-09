//! USB-HID operations

use std::time::Instant;

use log::trace;

pub(crate) const REPORT_ID: u8 = 0x00;

impl crate::Dap {
    /// Pushes the data into the HID buffer
    pub(crate) fn hid_push(&mut self, data: impl AsLeBytes) {
        data.as_le_bytes(|bytes| {
            let n = bytes.len();
            let cursor = usize::from(self.cursor);
            self.buffer[cursor..cursor + n].copy_from_slice(bytes);
            self.cursor += n as u16;
        });
    }

    /// Rewrites a byte in the HID buffer
    pub(crate) fn hid_rewrite(&mut self, i: u16, val: u8) {
        assert!(
            i < self.cursor,
            "attempt to modify unused part of the HID buffer"
        );
        self.buffer[usize::from(i)] = val;
    }

    /// Writes the contents of the HID buffer to the HID device
    pub(crate) fn hid_flush(&mut self) -> Result<(), anyhow::Error> {
        debug_assert_eq!(
            self.buffer[0], REPORT_ID,
            "first byte must be `REPORT_ID`"
        );

        let bytes = &self.buffer[..self.cursor.into()];
        let start = Instant::now();
        self.device.write(bytes)?;
        let end = Instant::now();
        trace!("HID <- <{} bytes> in {:?}", bytes.len(), end - start);
        self.cursor = 1;

        Ok(())
    }

    /// Reads `len` bytes from the HID device
    ///
    /// # Panics
    ///
    /// This function panics if
    ///
    /// - `hid_push` has been used but the HID buffer has not been drained
    /// - `len` exceeds the packet size supported by the target
    pub(crate) fn hid_read(
        &mut self,
        len: u16,
    ) -> Result<&[u8], anyhow::Error> {
        assert_eq!(self.cursor, 1, "HID buffer must be flushed before a read");
        assert!(
            len <= self.packet_size,
            "requested HID exceeds the target's packet size"
        );

        let buf = &mut self.buffer[1..];
        let start = Instant::now();
        let n = self.device.read(&mut buf[..usize::from(len)])?;
        assert_eq!(n, usize::from(len), "read less bytes than requested");
        let bytes = &buf[..n];
        let end = Instant::now();
        trace!("HID -> <{} bytes> in {:?}", bytes.len(), end - start);
        Ok(bytes)
    }
}

pub(crate) trait AsLeBytes {
    fn as_le_bytes(&self, f: impl FnOnce(&[u8]));
}

impl<T> AsLeBytes for &'_ T
where
    T: AsLeBytes + ?Sized,
{
    fn as_le_bytes(&self, f: impl FnOnce(&[u8])) {
        T::as_le_bytes(self, f)
    }
}

impl AsLeBytes for [u8] {
    fn as_le_bytes(&self, f: impl FnOnce(&[u8])) {
        f(self)
    }
}

impl AsLeBytes for u8 {
    fn as_le_bytes(&self, f: impl FnOnce(&[u8])) {
        f(&[*self])
    }
}

impl AsLeBytes for u16 {
    fn as_le_bytes(&self, f: impl FnOnce(&[u8])) {
        f(&self.to_le_bytes())
    }
}

impl AsLeBytes for u32 {
    fn as_le_bytes(&self, f: impl FnOnce(&[u8])) {
        f(&self.to_le_bytes())
    }
}
