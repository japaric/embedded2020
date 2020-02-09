use core::fmt;

pub struct Hex<T>(pub T);

impl fmt::Display for Hex<u8> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

impl fmt::Display for Hex<u64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 <= 0xff {
            Hex(self.0 as u8).fmt(f)
        } else if self.0 <= 0xffff {
            write!(f, "{:#06x}", self.0)
        } else if self.0 <= 0xffff_ffff {
            write!(f, "{:#06x}_{:04x}", self.0 >> 16, self.0 as u16)
        } else {
            unimplemented!()
        }
    }
}
