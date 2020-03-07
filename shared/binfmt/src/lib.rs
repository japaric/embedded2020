#![deny(warnings)]
#![no_std]

pub mod derive;
mod impls;
mod util;

use proc_macro_hack::proc_macro_hack;

#[doc(hidden)]
#[proc_macro_hack]
pub use binfmt_macros::binwriteln_;

/// Writes a `binfmt` encoded format string plus arguments to the given writer
#[proc_macro_hack]
pub use binfmt_macros::binwriteln;

macro_rules! u8_enum {
    ($($ident:ident = $expr:expr,)+) => {
        #[derive(Clone, Copy, PartialEq)]
        #[repr(u8)]
        pub enum Tag {
            $($ident = $expr,)+
        }

        impl Tag {
            pub fn from(byte: u8) -> Option<Tag> {
                Some(match byte {
                    $($expr => Tag::$ident,)+
                    _ => return None,
                })
            }
        }
    }
}

u8_enum! {
    // NOTE values must match the values in the `Level` enum
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
    Footprint = 5,
    Unsigned = 6,
    Signed = 7,
    F32 = 8,
    Pointer = 9,
    Register = 10,
}

#[repr(u8)]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

#[allow(non_camel_case_types)]
pub trait binDebug {
    fn fmt(&self, f: &mut impl binWrite);
}

const CONTINUE: u8 = 1 << 7;

#[allow(non_camel_case_types)]
pub trait binWrite: Sized {
    fn write_byte(&mut self, byte: u8);

    fn write(&mut self, bytes: &[u8]);

    fn leb128_write(&mut self, mut word: u32) {
        loop {
            let mut byte = (word & 0x7f) as u8;
            word >>= 7;

            if word != 0 {
                byte |= CONTINUE;
            }
            self.write_byte(byte);

            if word == 0 {
                return;
            }
        }
    }

    fn write_sym(&mut self, sym: *const u8) {
        let sym = sym as u16;
        if sym < 127 {
            self.write_byte(sym as u8);
        } else {
            self.write_byte(sym as u8 | CONTINUE);
            self.write_byte((sym >> 7) as u8);
        }
    }
}
