#![deny(warnings)]
#![no_std]

pub mod derive;
mod impls;
mod util;

use proc_macro_hack::proc_macro_hack;

// add only if needed
// #[proc_macro_hack]
// pub use binfmt_macros::binwrite;

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

#[allow(non_camel_case_types)]
pub trait binWrite: Sized {
    fn write(&mut self, bytes: &[u8]);

    fn write_footprint(&mut self, sym: *const u8) {
        if cfg!(target_pointer_width = "32") {
            util::binfmt_u32(Tag::Footprint as u8, sym as u32, self)
        } else {
            todo!()
        }
    }

    fn log(&mut self, level: Level, timestamp: u32) {
        util::binfmt_u32(level as u8, timestamp, self)
    }
}

#[allow(deprecated)]
unsafe fn uninitialized<T>() -> T {
    core::mem::uninitialized()
}
