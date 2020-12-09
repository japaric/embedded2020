#![allow(intra_doc_link_resolution_failure)]
#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![doc = "Cortex-M register API\n\n# References\n- ARMv7-M Architecture Reference Manual (ARM DDI 0403E.b)"]
#![no_std]
use core::marker::PhantomData;
#[doc = r" An instance of a peripheral"]
pub trait Peripheral {
    #[doc = r" The base address of the peripheral instance"]
    fn base_address() -> usize;
}
#[allow(dead_code)]
struct NotSendOrSync {
    inner: PhantomData<*mut ()>,
}
#[allow(dead_code)]
impl NotSendOrSync {
    fn new() -> Self {
        Self { inner: PhantomData }
    }
}
#[allow(non_camel_case_types)]
#[cfg(feature = "DCB")]
#[doc = "Debug Control Block"]
pub type DCB = dcb::Registers;
#[cfg(feature = "DCB")]
#[doc = "Debug Control Block"]
pub mod dcb {
    #[allow(non_camel_case_types)]
    #[doc = "Debug Halting Control and Status Register"]
    pub type DHCSR = dhcsr::Register;
    #[doc = "Debug Halting Control and Status Register"]
    pub mod dhcsr {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                super::BASE_ADDRESS as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub fn zero(&self) {
                unsafe {
                    Self::address().write_volatile(0);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(R, &mut W) -> &mut W) {
                let r = self.read();
                let mut w = r.into();
                f(r, &mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bit 0)"]
            pub fn C_DEBUGEN(self) -> u8 {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 1)"]
            pub fn C_HALT(self) -> u8 {
                const OFFSET: u8 = 1;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 2)"]
            pub fn C_STEP(self) -> u8 {
                const OFFSET: u8 = 2;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 3)"]
            pub fn C_MASKINTS(self) -> u8 {
                const OFFSET: u8 = 3;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 5)"]
            pub fn C_SNAPSTALL(self) -> u8 {
                const OFFSET: u8 = 5;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 16)"]
            pub fn S_REGRDY(self) -> u8 {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 17)"]
            pub fn S_HALT(self) -> u8 {
                const OFFSET: u8 = 17;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 18)"]
            pub fn S_SLEEP(self) -> u8 {
                const OFFSET: u8 = 18;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 19)"]
            pub fn S_LOCKUP(self) -> u8 {
                const OFFSET: u8 = 19;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 24)"]
            pub fn S_RETIRE_ST(self) -> u8 {
                const OFFSET: u8 = 24;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 25)"]
            pub fn S_RESET_ST(self) -> u8 {
                const OFFSET: u8 = 25;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "DHCSR {{ S_RESET_ST: {25}, S_RETIRE_ST: {24}, S_LOCKUP: {19}, S_SLEEP: {18}, S_HALT: {17}, S_REGRDY: {16}, C_SNAPSTALL: {5}, C_MASKINTS: {3}, C_STEP: {2}, C_HALT: {1}, C_DEBUGEN: {0} }}@11640593454652373321"]
                #[link_section = ".binfmt.DHCSR {{ S_RESET_ST: {25}, S_RETIRE_ST: {24}, S_LOCKUP: {19}, S_SLEEP: {18}, S_HALT: {17}, S_REGRDY: {16}, C_SNAPSTALL: {5}, C_MASKINTS: {3}, C_STEP: {2}, C_HALT: {1}, C_DEBUGEN: {0} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bit 0)"]
            #[allow(non_snake_case)]
            pub fn C_DEBUGEN(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 1)"]
            #[allow(non_snake_case)]
            pub fn C_HALT(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 1;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 2)"]
            #[allow(non_snake_case)]
            pub fn C_STEP(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 2;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 3)"]
            #[allow(non_snake_case)]
            pub fn C_MASKINTS(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 3;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 5)"]
            #[allow(non_snake_case)]
            pub fn C_SNAPSTALL(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 5;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bits 16..=32)"]
            #[allow(non_snake_case)]
            pub fn DBGKEY(&mut self, val: u16) -> &mut Self {
                const OFFSET: u8 = 16;
                const MASK: u16 = 0xffff;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
        impl From<R> for W {
            fn from(r: R) -> W {
                W {
                    inner: r.inner & !(0x030f_0000),
                }
            }
        }
        impl W {
            #[doc = r" Copies the contents of `R`"]
            pub fn copy(&mut self, r: R) -> &mut Self {
                *self = r.into();
                self
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Debug Core Register Selector Register"]
    pub type DCRSR = dcrsr::Register;
    #[doc = "Debug Core Register Selector Register"]
    pub mod dcrsr {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x04) as *mut _
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub fn zero(&self) {
                unsafe {
                    Self::address().write_volatile(0);
                }
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bits 0..=7)"]
            #[allow(non_snake_case)]
            pub fn REGSEL(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x7f;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 16)"]
            #[allow(non_snake_case)]
            pub fn REGWnR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Debug Core Register Data Register"]
    pub type DCRDR = dcrdr::Register;
    #[doc = "Debug Core Register Data Register"]
    pub mod dcrdr {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x08) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub unsafe fn write(&self, bits: u32) {
                {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub unsafe fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Debug Exception and Monitor Control Register"]
    pub type DEMCR = demcr::Register;
    #[doc = "Debug Exception and Monitor Control Register"]
    pub mod demcr {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x0c) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub fn zero(&self) {
                unsafe {
                    Self::address().write_volatile(0);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(R, &mut W) -> &mut W) {
                let r = self.read();
                let mut w = r.into();
                f(r, &mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bit 0)"]
            pub fn VC_CORERESET(self) -> u8 {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 4)"]
            pub fn VC_MMERR(self) -> u8 {
                const OFFSET: u8 = 4;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 5)"]
            pub fn VC_NOCPERR(self) -> u8 {
                const OFFSET: u8 = 5;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 6)"]
            pub fn VC_CHKERR(self) -> u8 {
                const OFFSET: u8 = 6;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 7)"]
            pub fn VC_STATERR(self) -> u8 {
                const OFFSET: u8 = 7;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 8)"]
            pub fn VC_BUSERR(self) -> u8 {
                const OFFSET: u8 = 8;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 9)"]
            pub fn VC_INTERR(self) -> u8 {
                const OFFSET: u8 = 9;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 10)"]
            pub fn VC_HARDERR(self) -> u8 {
                const OFFSET: u8 = 10;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 16)"]
            pub fn MON_EN(self) -> u8 {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 17)"]
            pub fn MON_PEND(self) -> u8 {
                const OFFSET: u8 = 17;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 18)"]
            pub fn MON_STEP(self) -> u8 {
                const OFFSET: u8 = 18;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 19)"]
            pub fn MON_REQ(self) -> u8 {
                const OFFSET: u8 = 19;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 24) Enables the DWT and ITM:\n0: DWT and ITM are disabled.\n1: DWT and ITM are enabled."]
            pub fn TRCENA(self) -> u8 {
                const OFFSET: u8 = 24;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "DEMCR {{ TRCENA: {24}, MON_REQ: {19}, MON_STEP: {18}, MON_PEND: {17}, MON_EN: {16}, VC_HARDERR: {10}, VC_INTERR: {9}, VC_BUSERR: {8}, VC_STATERR: {7}, VC_CHKERR: {6}, VC_NOCPERR: {5}, VC_MMERR: {4}, VC_CORERESET: {0} }}@1575267132128071719"]
                #[link_section = ".binfmt.DEMCR {{ TRCENA: {24}, MON_REQ: {19}, MON_STEP: {18}, MON_PEND: {17}, MON_EN: {16}, VC_HARDERR: {10}, VC_INTERR: {9}, VC_BUSERR: {8}, VC_STATERR: {7}, VC_CHKERR: {6}, VC_NOCPERR: {5}, VC_MMERR: {4}, VC_CORERESET: {0} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bit 0)"]
            #[allow(non_snake_case)]
            pub fn VC_CORERESET(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 4)"]
            #[allow(non_snake_case)]
            pub fn VC_MMERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 4;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 5)"]
            #[allow(non_snake_case)]
            pub fn VC_NOCPERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 5;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 6)"]
            #[allow(non_snake_case)]
            pub fn VC_CHKERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 6;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 7)"]
            #[allow(non_snake_case)]
            pub fn VC_STATERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 7;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 8)"]
            #[allow(non_snake_case)]
            pub fn VC_BUSERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 8;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 9)"]
            #[allow(non_snake_case)]
            pub fn VC_INTERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 9;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 10)"]
            #[allow(non_snake_case)]
            pub fn VC_HARDERR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 10;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 16)"]
            #[allow(non_snake_case)]
            pub fn MON_EN(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 17)"]
            #[allow(non_snake_case)]
            pub fn MON_PEND(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 17;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 18)"]
            #[allow(non_snake_case)]
            pub fn MON_STEP(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 18;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 19)"]
            #[allow(non_snake_case)]
            pub fn MON_REQ(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 19;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 24) Enables the DWT and ITM:\n0: DWT and ITM are disabled.\n1: DWT and ITM are enabled."]
            #[allow(non_snake_case)]
            pub fn TRCENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 24;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
        impl From<R> for W {
            fn from(r: R) -> W {
                W { inner: r.inner }
            }
        }
        impl W {
            #[doc = r" Copies the contents of `R`"]
            pub fn copy(&mut self, r: R) -> &mut Self {
                *self = r.into();
                self
            }
        }
    }
    use core::sync::atomic::{AtomicBool, Ordering};
    const BASE_ADDRESS: usize = 0xe000_edf0;
    #[allow(non_snake_case)]
    #[doc = "Singleton handle to the DCB registers"]
    pub struct Registers {
        #[doc = "Debug Halting Control and Status Register"]
        pub DHCSR: DHCSR,
        #[doc = "Debug Core Register Selector Register"]
        pub DCRSR: DCRSR,
        #[doc = "Debug Core Register Data Register"]
        pub DCRDR: DCRDR,
        #[doc = "Debug Exception and Monitor Control Register"]
        pub DEMCR: DEMCR,
    }
    unsafe impl Send for Registers {}
    impl Registers {
        #[doc = r" # Safety"]
        #[doc = r" Singleton"]
        unsafe fn new() -> Self {
            Self {
                DHCSR: DHCSR::new(),
                DCRSR: DCRSR::new(),
                DCRDR: DCRDR::new(),
                DEMCR: DEMCR::new(),
            }
        }
        fn taken() -> &'static AtomicBool {
            static TAKEN: AtomicBool = AtomicBool::new(false);
            &TAKEN
        }
        #[doc = r" Grants temporary access to the peripheral, without checking if it has already been"]
        #[doc = r" taken"]
        #[inline(always)]
        pub fn borrow_unchecked<T>(f: impl FnOnce(&Self) -> T) -> T {
            f(unsafe { &Self::new() })
        }
        #[doc = r" Seals the peripheral making it impossible to `take` it"]
        pub fn seal() {
            Self::taken().store(true, Ordering::Relaxed)
        }
        #[doc = r" Takes ownership of the peripheral"]
        #[doc = r""]
        #[doc = r" This constructor returns the `Some` variant only once"]
        pub fn take() -> Option<Self> {
            let taken = Self::taken();
            if taken
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                Some(unsafe { Self::new() })
            } else {
                None
            }
        }
    }
}
#[allow(non_camel_case_types)]
#[cfg(feature = "DWT")]
#[doc = "Data Watchpoint and Trace"]
pub type DWT = dwt::Registers;
#[cfg(feature = "DWT")]
#[doc = "Data Watchpoint and Trace"]
pub mod dwt {
    #[allow(non_camel_case_types)]
    #[doc = "Control register"]
    pub type CTRL = ctrl::Register;
    #[doc = "Control register"]
    pub mod ctrl {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                super::BASE_ADDRESS as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub fn zero(&self) {
                unsafe {
                    Self::address().write_volatile(0);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(R, &mut W) -> &mut W) {
                let r = self.read();
                let mut w = r.into();
                f(r, &mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bit 0) Enables the cycle counter.\n0: Counter disabled.\n1: Counter enabled."]
            pub fn CYCCNTENA(self) -> u8 {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 1)"]
            pub fn POSTPRESET(self) -> u8 {
                const OFFSET: u8 = 1;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 5..=9)"]
            pub fn POSTINIT(self) -> u8 {
                const OFFSET: u8 = 5;
                const MASK: u8 = 0x0f;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 9)"]
            pub fn CYCTAP(self) -> u8 {
                const OFFSET: u8 = 9;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 10..=12)"]
            pub fn SYNCTAP(self) -> u8 {
                const OFFSET: u8 = 10;
                const MASK: u8 = 0x03;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 12)"]
            pub fn PCSAMPLENA(self) -> u8 {
                const OFFSET: u8 = 12;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 16)"]
            pub fn EXCTRCENA(self) -> u8 {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 17)"]
            pub fn CPIEVTENA(self) -> u8 {
                const OFFSET: u8 = 17;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 18)"]
            pub fn EXCEVTENA(self) -> u8 {
                const OFFSET: u8 = 18;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 19)"]
            pub fn SLEEPEVTENA(self) -> u8 {
                const OFFSET: u8 = 19;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 20)"]
            pub fn LSUEVTENA(self) -> u8 {
                const OFFSET: u8 = 20;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 21)"]
            pub fn FOLDEVTENA(self) -> u8 {
                const OFFSET: u8 = 21;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 22)"]
            pub fn CYCEVTENA(self) -> u8 {
                const OFFSET: u8 = 22;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 24)"]
            pub fn NOPRFCNT(self) -> u8 {
                const OFFSET: u8 = 24;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 25)"]
            pub fn NOCYCCNT(self) -> u8 {
                const OFFSET: u8 = 25;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 26)"]
            pub fn NOEXTTRIG(self) -> u8 {
                const OFFSET: u8 = 26;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 27)"]
            pub fn NOTRCPKT(self) -> u8 {
                const OFFSET: u8 = 27;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 28..=32)"]
            pub fn NUMCOMP(self) -> u8 {
                const OFFSET: u8 = 28;
                const MASK: u8 = 0x0f;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "CTRL {{ NUMCOMP: {28:32}, NOTRCPKT: {27}, NOEXTTRIG: {26}, NOCYCCNT: {25}, NOPRFCNT: {24}, CYCEVTENA: {22}, FOLDEVTENA: {21}, LSUEVTENA: {20}, SLEEPEVTENA: {19}, EXCEVTENA: {18}, CPIEVTENA: {17}, EXCTRCENA: {16}, PCSAMPLENA: {12}, SYNCTAP: {10:12}, CYCTAP: {9}, POSTINIT: {5:9}, POSTPRESET: {1}, CYCCNTENA: {0} }}@3702287509423000095"]
                #[link_section = ".binfmt.CTRL {{ NUMCOMP: {28:32}, NOTRCPKT: {27}, NOEXTTRIG: {26}, NOCYCCNT: {25}, NOPRFCNT: {24}, CYCEVTENA: {22}, FOLDEVTENA: {21}, LSUEVTENA: {20}, SLEEPEVTENA: {19}, EXCEVTENA: {18}, CPIEVTENA: {17}, EXCTRCENA: {16}, PCSAMPLENA: {12}, SYNCTAP: {10:12}, CYCTAP: {9}, POSTINIT: {5:9}, POSTPRESET: {1}, CYCCNTENA: {0} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bit 0) Enables the cycle counter.\n0: Counter disabled.\n1: Counter enabled."]
            #[allow(non_snake_case)]
            pub fn CYCCNTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 1)"]
            #[allow(non_snake_case)]
            pub fn POSTPRESET(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 1;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bits 5..=9)"]
            #[allow(non_snake_case)]
            pub fn POSTINIT(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 5;
                const MASK: u8 = 0x0f;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 9)"]
            #[allow(non_snake_case)]
            pub fn CYCTAP(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 9;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bits 10..=12)"]
            #[allow(non_snake_case)]
            pub fn SYNCTAP(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 10;
                const MASK: u8 = 0x03;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 12)"]
            #[allow(non_snake_case)]
            pub fn PCSAMPLENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 12;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 16)"]
            #[allow(non_snake_case)]
            pub fn EXCTRCENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 17)"]
            #[allow(non_snake_case)]
            pub fn CPIEVTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 17;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 18)"]
            #[allow(non_snake_case)]
            pub fn EXCEVTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 18;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 19)"]
            #[allow(non_snake_case)]
            pub fn SLEEPEVTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 19;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 20)"]
            #[allow(non_snake_case)]
            pub fn LSUEVTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 20;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 21)"]
            #[allow(non_snake_case)]
            pub fn FOLDEVTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 21;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 22)"]
            #[allow(non_snake_case)]
            pub fn CYCEVTENA(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 22;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
        impl From<R> for W {
            fn from(r: R) -> W {
                W {
                    inner: r.inner & !(0xff00_0000),
                }
            }
        }
        impl W {
            #[doc = r" Copies the contents of `R`"]
            pub fn copy(&mut self, r: R) -> &mut Self {
                *self = r.into();
                self
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Cycle Count register"]
    pub type CYCCNT = cyccnt::Register;
    #[doc = "Cycle Count register"]
    pub mod cyccnt {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x04) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub fn write(&self, bits: u32) {
                unsafe {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    use core::sync::atomic::{AtomicBool, Ordering};
    const BASE_ADDRESS: usize = 0xe000_1000;
    #[allow(non_snake_case)]
    #[doc = "Singleton handle to the DWT registers"]
    pub struct Registers {
        #[doc = "Control register"]
        pub CTRL: CTRL,
        #[doc = "Cycle Count register"]
        pub CYCCNT: CYCCNT,
    }
    unsafe impl Send for Registers {}
    impl Registers {
        #[doc = r" # Safety"]
        #[doc = r" Singleton"]
        unsafe fn new() -> Self {
            Self {
                CTRL: CTRL::new(),
                CYCCNT: CYCCNT::new(),
            }
        }
        fn taken() -> &'static AtomicBool {
            static TAKEN: AtomicBool = AtomicBool::new(false);
            &TAKEN
        }
        #[doc = r" Grants temporary access to the peripheral, without checking if it has already been"]
        #[doc = r" taken"]
        #[inline(always)]
        pub fn borrow_unchecked<T>(f: impl FnOnce(&Self) -> T) -> T {
            f(unsafe { &Self::new() })
        }
        #[doc = r" Seals the peripheral making it impossible to `take` it"]
        pub fn seal() {
            Self::taken().store(true, Ordering::Relaxed)
        }
        #[doc = r" Takes ownership of the peripheral"]
        #[doc = r""]
        #[doc = r" This constructor returns the `Some` variant only once"]
        pub fn take() -> Option<Self> {
            let taken = Self::taken();
            if taken
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                Some(unsafe { Self::new() })
            } else {
                None
            }
        }
    }
}
#[allow(non_camel_case_types)]
#[cfg(feature = "NVIC")]
#[doc = "Nested Vector Interrupt Controller"]
pub type NVIC = nvic::Registers;
#[cfg(feature = "NVIC")]
#[doc = "Nested Vector Interrupt Controller"]
pub mod nvic {
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Set-Enable Register 0"]
    pub type ISER0 = iser0::Register;
    #[doc = "Interrupt Set-Enable Register 0"]
    pub mod iser0 {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                super::BASE_ADDRESS as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub unsafe fn write(&self, bits: u32) {
                {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub unsafe fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Set-Enable Register 1"]
    pub type ISER1 = iser1::Register;
    #[doc = "Interrupt Set-Enable Register 1"]
    pub mod iser1 {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x04) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub unsafe fn write(&self, bits: u32) {
                {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub unsafe fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Clear-Enable Register 0"]
    pub type ICER0 = icer0::Register;
    #[doc = "Interrupt Clear-Enable Register 0"]
    pub mod icer0 {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x80) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub fn write(&self, bits: u32) {
                unsafe {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Clear-Enable Register 1"]
    pub type ICER1 = icer1::Register;
    #[doc = "Interrupt Clear-Enable Register 1"]
    pub mod icer1 {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x84) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub fn write(&self, bits: u32) {
                unsafe {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Set-Pending Register 0"]
    pub type ISPR0 = ispr0::Register;
    #[doc = "Interrupt Set-Pending Register 0"]
    pub mod ispr0 {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x0100) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub fn write(&self, bits: u32) {
                unsafe {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Set-Pending Register 1"]
    pub type ISPR1 = ispr1::Register;
    #[doc = "Interrupt Set-Pending Register 1"]
    pub mod ispr1 {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x0104) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> u32 {
                unsafe { Self::address().read_volatile() }
            }
            #[doc = r" Writes `bits` to the register in a single, volatile instruction"]
            pub fn write(&self, bits: u32) {
                unsafe {
                    Self::address().write_volatile(bits);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(u32) -> u32) {
                self.write(f(self.read()))
            }
        }
    }
    use core::sync::atomic::{AtomicBool, Ordering};
    const BASE_ADDRESS: usize = 0xe000_e100;
    #[allow(non_snake_case)]
    #[doc = "Singleton handle to the NVIC registers"]
    pub struct Registers {
        #[doc = "Interrupt Set-Enable Register 0"]
        pub ISER0: ISER0,
        #[doc = "Interrupt Set-Enable Register 1"]
        pub ISER1: ISER1,
        #[doc = "Interrupt Clear-Enable Register 0"]
        pub ICER0: ICER0,
        #[doc = "Interrupt Clear-Enable Register 1"]
        pub ICER1: ICER1,
        #[doc = "Interrupt Set-Pending Register 0"]
        pub ISPR0: ISPR0,
        #[doc = "Interrupt Set-Pending Register 1"]
        pub ISPR1: ISPR1,
    }
    unsafe impl Send for Registers {}
    impl Registers {
        #[doc = r" # Safety"]
        #[doc = r" Singleton"]
        unsafe fn new() -> Self {
            Self {
                ISER0: ISER0::new(),
                ISER1: ISER1::new(),
                ICER0: ICER0::new(),
                ICER1: ICER1::new(),
                ISPR0: ISPR0::new(),
                ISPR1: ISPR1::new(),
            }
        }
        fn taken() -> &'static AtomicBool {
            static TAKEN: AtomicBool = AtomicBool::new(false);
            &TAKEN
        }
        #[doc = r" Grants temporary access to the peripheral, without checking if it has already been"]
        #[doc = r" taken"]
        #[inline(always)]
        pub fn borrow_unchecked<T>(f: impl FnOnce(&Self) -> T) -> T {
            f(unsafe { &Self::new() })
        }
        #[doc = r" Seals the peripheral making it impossible to `take` it"]
        pub fn seal() {
            Self::taken().store(true, Ordering::Relaxed)
        }
        #[doc = r" Takes ownership of the peripheral"]
        #[doc = r""]
        #[doc = r" This constructor returns the `Some` variant only once"]
        pub fn take() -> Option<Self> {
            let taken = Self::taken();
            if taken
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                Some(unsafe { Self::new() })
            } else {
                None
            }
        }
    }
}
#[allow(non_camel_case_types)]
#[cfg(feature = "SCB")]
#[doc = "System Control Block"]
pub type SCB = scb::Registers;
#[cfg(feature = "SCB")]
#[doc = "System Control Block"]
pub mod scb {
    #[allow(non_camel_case_types)]
    #[doc = "CPUID Base register"]
    pub type CPUID = cpuid::Register;
    #[doc = "CPUID Base register"]
    pub mod cpuid {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *const u32 {
                super::BASE_ADDRESS as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bits 0..=4)"]
            pub fn REVISION(self) -> u8 {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x0f;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 4..=16)"]
            pub fn PARTNO(self) -> u16 {
                const OFFSET: u8 = 4;
                const MASK: u16 = 0x0fff;
                ((self.inner >> OFFSET) as u16) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 16..=20)"]
            pub fn ARCHITECTURE(self) -> u8 {
                const OFFSET: u8 = 16;
                const MASK: u8 = 0x0f;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 20..=24)"]
            pub fn VARIANT(self) -> u8 {
                const OFFSET: u8 = 20;
                const MASK: u8 = 0x0f;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 24..=32)"]
            pub fn IMPLEMENTER(self) -> u8 {
                const OFFSET: u8 = 24;
                const MASK: u8 = 0xff;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "CPUID {{ IMPLEMENTER: {24:32}, VARIANT: {20:24}, ARCHITECTURE: {16:20}, PARTNO: {4:16}, REVISION: {0:4} }}@3829545955687387708"]
                #[link_section = ".binfmt.CPUID {{ IMPLEMENTER: {24:32}, VARIANT: {20:24}, ARCHITECTURE: {16:20}, PARTNO: {4:16}, REVISION: {0:4} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Interrupt Control and State Register"]
    pub type ICSR = icsr::Register;
    #[doc = "Interrupt Control and State Register"]
    pub mod icsr {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x04) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub fn zero(&self) {
                unsafe {
                    Self::address().write_volatile(0);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(R, &mut W) -> &mut W) {
                let r = self.read();
                let mut w = r.into();
                f(r, &mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bit 26)"]
            pub fn PENDSTSET(self) -> u8 {
                const OFFSET: u8 = 26;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 28)"]
            pub fn PENDSVSET(self) -> u8 {
                const OFFSET: u8 = 28;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 31)"]
            pub fn NMIPENDSET(self) -> u8 {
                const OFFSET: u8 = 31;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 0..=9) The vector table index of the exception currently being executed.\n0: Thread mode\n!0: Exception context"]
            pub fn VECTACTIVE(self) -> u16 {
                const OFFSET: u8 = 0;
                const MASK: u16 = 0x01ff;
                ((self.inner >> OFFSET) as u16) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 11)"]
            pub fn RETTOBASE(self) -> u8 {
                const OFFSET: u8 = 11;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 12..=21)"]
            pub fn VECTPENDING(self) -> u16 {
                const OFFSET: u8 = 12;
                const MASK: u16 = 0x01ff;
                ((self.inner >> OFFSET) as u16) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 22)"]
            pub fn ISRPENDING(self) -> u8 {
                const OFFSET: u8 = 22;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 23)"]
            pub fn ISRPREEMPT(self) -> u8 {
                const OFFSET: u8 = 23;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "ICSR {{ ISRPREEMPT: {23}, ISRPENDING: {22}, VECTPENDING: {12:21}, RETTOBASE: {11}, VECTACTIVE: {0:9}, NMIPENDSET: {31}, PENDSVSET: {28}, PENDSTSET: {26} }}@17294273114561865096"]
                #[link_section = ".binfmt.ICSR {{ ISRPREEMPT: {23}, ISRPENDING: {22}, VECTPENDING: {12:21}, RETTOBASE: {11}, VECTACTIVE: {0:9}, NMIPENDSET: {31}, PENDSVSET: {28}, PENDSTSET: {26} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bit 26)"]
            #[allow(non_snake_case)]
            pub fn PENDSTSET(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 26;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 28)"]
            #[allow(non_snake_case)]
            pub fn PENDSVSET(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 28;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 31)"]
            #[allow(non_snake_case)]
            pub fn NMIPENDSET(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 31;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 25)"]
            #[allow(non_snake_case)]
            pub fn PENDSTCLR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 25;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 27)"]
            #[allow(non_snake_case)]
            pub fn PENDSVCLR(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 27;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
        impl From<R> for W {
            fn from(r: R) -> W {
                W {
                    inner: r.inner & !(0x00df_f9ff),
                }
            }
        }
        impl W {
            #[doc = r" Copies the contents of `R`"]
            pub fn copy(&mut self, r: R) -> &mut Self {
                *self = r.into();
                self
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Vector Table Offset Register"]
    pub type VTOR = vtor::Register;
    #[doc = "Vector Table Offset Register"]
    pub mod vtor {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x08) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub unsafe fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub unsafe fn zero(&self) {
                {
                    Self::address().write_volatile(0);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub unsafe fn rmw(&self, f: impl FnOnce(R, &mut W) -> &mut W) {
                let r = self.read();
                let mut w = r.into();
                f(r, &mut w);
                {
                    Self::address().write_volatile(w.into());
                }
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bits 7..=32)"]
            pub fn TBLOFF(self) -> u32 {
                const OFFSET: u8 = 7;
                const MASK: u32 = 0x01ff_ffff;
                ((self.inner >> OFFSET) as u32) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "VTOR {{ TBLOFF: {7:32} }}@1346214485063033802"]
                #[link_section = ".binfmt.VTOR {{ TBLOFF: {7:32} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bits 7..=32)"]
            #[allow(non_snake_case)]
            pub fn TBLOFF(&mut self, val: u32) -> &mut Self {
                const OFFSET: u8 = 7;
                const MASK: u32 = 0x01ff_ffff;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
        impl From<R> for W {
            fn from(r: R) -> W {
                W { inner: r.inner }
            }
        }
        impl W {
            #[doc = r" Copies the contents of `R`"]
            pub fn copy(&mut self, r: R) -> &mut Self {
                *self = r.into();
                self
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[doc = "Application Interrupt and Reset Control Register"]
    pub type AIRCR = aircr::Register;
    #[doc = "Application Interrupt and Reset Control Register"]
    pub mod aircr {
        use crate::NotSendOrSync;
        #[doc = r" Singleton handle to the register"]
        pub struct Register {
            _not_send_or_sync: NotSendOrSync,
        }
        impl Register {
            #[doc = r" # Safety"]
            #[doc = r" Singleton"]
            pub(crate) unsafe fn new() -> Self {
                Self {
                    _not_send_or_sync: NotSendOrSync::new(),
                }
            }
            #[doc = r" Returns the address of this register"]
            pub fn address() -> *mut u32 {
                (super::BASE_ADDRESS + 0x0c) as *mut _
            }
            #[doc = r" Reads the contents of the register in a single, volatile instruction"]
            pub fn read(&self) -> R {
                R::from(unsafe { Self::address().read_volatile() })
            }
            #[doc = r" Writes the bits set by `f` to the register in a single, volatile instruction"]
            #[inline(always)]
            pub fn write(&self, f: impl FnOnce(&mut W) -> &mut W) {
                let mut w = W::zero();
                f(&mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
            #[doc = r" Writes zeros to the register"]
            #[inline(always)]
            pub fn zero(&self) {
                unsafe {
                    Self::address().write_volatile(0);
                }
            }
            #[doc = r" Updates the contents of the register using the closure `f`"]
            #[doc = r""]
            #[doc = r" This performs a `read` operation followed by a `write` operation"]
            #[inline(always)]
            pub fn rmw(&self, f: impl FnOnce(R, &mut W) -> &mut W) {
                let r = self.read();
                let mut w = r.into();
                f(r, &mut w);
                unsafe {
                    Self::address().write_volatile(w.into());
                }
            }
        }
        #[doc = r" View into the readable bitfields"]
        #[derive(Clone, Copy)]
        pub struct R {
            inner: u32,
        }
        impl From<u32> for R {
            fn from(bits: u32) -> Self {
                R { inner: bits }
            }
        }
        impl From<R> for u32 {
            fn from(r: R) -> Self {
                r.inner
            }
        }
        impl R {
            #[allow(non_snake_case)]
            #[doc = "(Bit 2)"]
            pub fn SYSRESETREQ(self) -> u8 {
                const OFFSET: u8 = 2;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 8..=11)"]
            pub fn PRIGROUP(self) -> u8 {
                const OFFSET: u8 = 8;
                const MASK: u8 = 0x07;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bit 15)"]
            pub fn ENDIANNESS(self) -> u8 {
                const OFFSET: u8 = 15;
                const MASK: u8 = 0x01;
                ((self.inner >> OFFSET) as u8) & MASK
            }
            #[allow(non_snake_case)]
            #[doc = "(Bits 16..=32)"]
            pub fn VECTKEYSTAT(self) -> u16 {
                const OFFSET: u8 = 16;
                const MASK: u16 = 0xffff;
                ((self.inner >> OFFSET) as u16) & MASK
            }
            #[doc = r" Returns the non-reserved part of the register"]
            pub fn bits(self) -> u32 {
                self.inner as _
            }
        }
        #[cfg(feature = "binfmt")]
        impl binfmt::binDebug for R {
            fn fmt(&self, f: &mut impl binfmt::binWrite) {
                #[export_name = "AIRCR {{ VECTKEYSTAT: {16:32}, ENDIANNESS: {15}, PRIGROUP: {8:11}, SYSRESETREQ: {2} }}@5121947446505091337"]
                #[link_section = ".binfmt.AIRCR {{ VECTKEYSTAT: {16:32}, ENDIANNESS: {15}, PRIGROUP: {8:11}, SYSRESETREQ: {2} }}"]
                static SYM: u8 = 0;
                f.write_byte(binfmt::Tag::Register as u8);
                f.write_sym(&SYM);
                f.write(&(*self).bits().to_le_bytes());
            }
        }
        #[doc = r" View into the writable bitfields"]
        #[derive(Clone, Copy)]
        pub struct W {
            inner: u32,
        }
        impl From<W> for u32 {
            fn from(w: W) -> Self {
                w.inner
            }
        }
        impl W {
            #[doc = r" Writable view with all bitfields set to zero"]
            pub fn zero() -> W {
                W { inner: 0 }
            }
            #[doc = "(Bit 2)"]
            #[allow(non_snake_case)]
            pub fn SYSRESETREQ(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 2;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bits 8..=11)"]
            #[allow(non_snake_case)]
            pub fn PRIGROUP(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 8;
                const MASK: u8 = 0x07;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 0)"]
            #[allow(non_snake_case)]
            pub fn VECTRESET(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 0;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bit 1)"]
            #[allow(non_snake_case)]
            pub fn VECTCLRACTIVE(&mut self, val: u8) -> &mut Self {
                const OFFSET: u8 = 1;
                const MASK: u8 = 0x01;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
            #[doc = "(Bits 16..=32)"]
            #[allow(non_snake_case)]
            pub fn VECTKEY(&mut self, val: u16) -> &mut Self {
                const OFFSET: u8 = 16;
                const MASK: u16 = 0xffff;
                self.inner &= !((MASK as u32) << OFFSET);
                self.inner |= ((val & MASK) as u32) << OFFSET;
                self
            }
        }
        impl From<R> for W {
            fn from(r: R) -> W {
                W {
                    inner: r.inner & !(0xffff_8000),
                }
            }
        }
        impl W {
            #[doc = r" Copies the contents of `R`"]
            pub fn copy(&mut self, r: R) -> &mut Self {
                *self = r.into();
                self
            }
        }
    }
    use core::sync::atomic::{AtomicBool, Ordering};
    const BASE_ADDRESS: usize = 0xe000_ed00;
    #[allow(non_snake_case)]
    #[doc = "Singleton handle to the SCB registers"]
    pub struct Registers {
        #[doc = "CPUID Base register"]
        pub CPUID: CPUID,
        #[doc = "Interrupt Control and State Register"]
        pub ICSR: ICSR,
        #[doc = "Vector Table Offset Register"]
        pub VTOR: VTOR,
        #[doc = "Application Interrupt and Reset Control Register"]
        pub AIRCR: AIRCR,
    }
    unsafe impl Send for Registers {}
    impl Registers {
        #[doc = r" # Safety"]
        #[doc = r" Singleton"]
        unsafe fn new() -> Self {
            Self {
                CPUID: CPUID::new(),
                ICSR: ICSR::new(),
                VTOR: VTOR::new(),
                AIRCR: AIRCR::new(),
            }
        }
        fn taken() -> &'static AtomicBool {
            static TAKEN: AtomicBool = AtomicBool::new(false);
            &TAKEN
        }
        #[doc = r" Grants temporary access to the peripheral, without checking if it has already been"]
        #[doc = r" taken"]
        #[inline(always)]
        pub fn borrow_unchecked<T>(f: impl FnOnce(&Self) -> T) -> T {
            f(unsafe { &Self::new() })
        }
        #[doc = r" Seals the peripheral making it impossible to `take` it"]
        pub fn seal() {
            Self::taken().store(true, Ordering::Relaxed)
        }
        #[doc = r" Takes ownership of the peripheral"]
        #[doc = r""]
        #[doc = r" This constructor returns the `Some` variant only once"]
        pub fn take() -> Option<Self> {
            let taken = Self::taken();
            if taken
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                Some(unsafe { Self::new() })
            } else {
                None
            }
        }
    }
}
