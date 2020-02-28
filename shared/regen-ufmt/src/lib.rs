#![no_std]

use core::str;

unsafe fn bin(mut val: u8, buf: &mut [u8]) {
    for slot in buf.iter_mut().rev() {
        if val == 0 {
            return;
        }

        if val % 2 == 1 {
            *slot = b'1';
        }
        val /= 2;
    }
}

pub struct Bin1(pub u8);

impl ufmt::uDebug for Bin1 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        f.write_str(if self.0 == 0 { "0" } else { "1" })
    }
}

pub struct Bin2(pub u8);

impl ufmt::uDebug for Bin2 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'b', b'0', b'0'];
        unsafe {
            bin(self.0, &mut buf[2..]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Bin3(pub u8);

impl ufmt::uDebug for Bin3 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'b', b'0', b'0', b'0'];
        unsafe {
            bin(self.0, &mut buf[2..]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

unsafe fn hex(mut val: u16, buf: &mut [u8]) {
    for slot in buf.iter_mut().rev() {
        if val == 0 {
            return;
        }

        let digit = (val & 0xf) as u8;
        if digit < 10 {
            *slot = digit + b'0';
        } else {
            *slot = (digit - 10) + b'a';
        }
        val >>= 4;
    }
}
pub struct Hex1(pub u8);

impl ufmt::uDebug for Hex1 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'x', b'0'];
        unsafe {
            hex(self.0.into(), &mut buf[2..]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Hex2(pub u8);

impl ufmt::uDebug for Hex2 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'x', b'0', b'0'];
        unsafe {
            hex(self.0.into(), &mut buf[2..]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Hex3(pub u16);

impl ufmt::uDebug for Hex3 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'x', b'0', b'0', b'0'];
        unsafe {
            hex(self.0, &mut buf[2..]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Hex4(pub u16);

impl ufmt::uDebug for Hex4 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'x', b'0', b'0', b'0', b'0'];
        unsafe {
            hex(self.0, &mut buf[2..]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Hex6(pub u32);

impl ufmt::uDebug for Hex6 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf = [b'0', b'x', b'0', b'0', b'_', b'0', b'0', b'0', b'0'];
        unsafe {
            hex(self.0 as u16, &mut buf[5..]);
            hex((self.0 >> 16) as u16, &mut buf[2..4]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Hex7(pub u32);

impl ufmt::uDebug for Hex7 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf =
            [b'0', b'x', b'0', b'0', b'0', b'_', b'0', b'0', b'0', b'0'];
        unsafe {
            hex(self.0 as u16, &mut buf[6..]);
            hex((self.0 >> 16) as u16, &mut buf[2..5]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}

pub struct Hex8(pub u32);

impl ufmt::uDebug for Hex8 {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let mut buf =
            [b'0', b'x', b'0', b'0', b'0', b'0', b'_', b'0', b'0', b'0', b'0'];
        unsafe {
            hex(self.0 as u16, &mut buf[6..]);
            hex((self.0 >> 16) as u16, &mut buf[2..6]);
            f.write_str(str::from_utf8_unchecked(&buf))
        }
    }
}
