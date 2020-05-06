//! Cortex-M specific operations

use anyhow::bail;
use cm::{
    dcb::{dcrsr, demcr, dhcsr, DCRDR, DCRSR, DEMCR, DHCSR},
    scb::{aircr, AIRCR},
};
use log::info;

use crate::adiv5;

/// Cortex-M register that the DAP can read
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Register {
    /// R0
    R0 = 0b00000,

    /// R1
    R1 = 0b00001,

    /// R2
    R2 = 0b00010,

    /// R3
    R3 = 0b00011,

    /// R4
    R4 = 0b00100,

    /// R5
    R5 = 0b00101,

    /// R6
    R6 = 0b00110,

    /// R7
    R7 = 0b00111,

    /// R8
    R8 = 0b01000,

    /// R9
    R9 = 0b01001,

    /// R10
    R10 = 0b01010,

    /// R11
    R11 = 0b01011,

    /// R12
    R12 = 0b01100,

    /// Stack Pointer
    SP = 0b01101,

    /// Link register
    LR = 0b01110,

    /// Program Counter (AKA Debug Return Address)
    PC = 0b01111,

    /// Program Status Register
    XPSR = 0b10000,

    /// CONTROL - FAULTMASK - BASEPRI - PRIMASK
    CFBP = 0b10100,
}

impl Register {
    fn regsel(self) -> u8 {
        self as u8
    }
}

/// Debug Key
const DBGKEY: u16 = 0xA05F;

impl crate::Dap {
    /// [ARM Cortex-M] Halts a Cortex-M target
    pub fn halt(&mut self) -> Result<(), anyhow::Error> {
        info!("halting the target ...");

        self.set_debugen()?;

        let addr = DHCSR::address() as usize as u32;
        let dhcsr = dhcsr::R::from(self.memory_read_word(addr)?);
        if dhcsr.C_HALT() == 0 {
            let mut w = dhcsr::W::from(dhcsr);
            w.DBGKEY(DBGKEY).C_HALT(1).C_DEBUGEN(1);
            self.memory_write_word(addr, w.into())?;
            let dhcsr = dhcsr::R::from(self.memory_read_word(addr)?);

            if dhcsr.C_HALT() == 0 {
                bail!(
                    "failed to halt the target (DHCSR = {:#010x})",
                    u32::from(dhcsr)
                );
            }
        }

        info!("... target halted");

        Ok(())
    }

    /// [ARM Cortex-M] Resumes ARM Cortex-M execution
    pub fn resume(&mut self) -> Result<(), anyhow::Error> {
        if !self.is_halted()? {
            info!("target is not halted");
            return Ok(());
        }

        info!("resuming target ...");

        let addr = DHCSR::address() as usize as u32;
        let mut w = dhcsr::W::from(dhcsr::R::from(self.memory_read_word(addr)?));
        w.DBGKEY(DBGKEY).C_HALT(0).C_DEBUGEN(1);
        self.memory_write_word(addr, w.into())?;

        let dhcsr = dhcsr::R::from(self.memory_read_word(addr)?);

        if dhcsr.C_HALT() != 0 {
            bail!(
                "failed to resume the target (DHCSR = {:#010x})",
                u32::from(dhcsr)
            );
        }

        info!("... target resumed");

        Ok(())
    }

    /// [ARM Cortex-M] Checks if the target is currently halted
    pub fn is_halted(&mut self) -> Result<bool, anyhow::Error> {
        Ok(if self.debugen == Some(true) {
            dhcsr::R::from(self.memory_read_word(DHCSR::address() as usize as u32)?).S_HALT() != 0
        } else {
            false
        })
    }

    /// [ARM Cortex-M] Reads the specified target's core register
    pub fn read_core_register(&mut self, reg: Register) -> Result<u32, anyhow::Error> {
        const READ: u8 = 0;

        info!("reading Cortex-M register {:?} ... ", reg);

        let mut w = dcrsr::W::zero();
        w.REGWnR(READ).REGSEL(reg.regsel());
        self.memory_write_word(DCRSR::address() as usize as u32, w.into())?;

        loop {
            let dhcsr = dhcsr::R::from(self.memory_read_word(DHCSR::address() as usize as u32)?);
            if dhcsr.S_REGRDY() != 0 {
                break;
            }
            self.brief_sleep();
        }

        let word = self.memory_read_word(DCRDR::address() as usize as u32)?;

        info!("{:?} -> {:#010x}", reg, word);

        Ok(word)
    }

    /// [ARM Cortex-M] Requests a system reset
    pub fn sysresetreq(&mut self, halt: bool) -> Result<(), anyhow::Error> {
        /// Vector Key
        const VECTKEY: u16 = 0x05fa;

        info!("resetting the target with SYSRESETREQ ...");

        self.set_debugen()?;

        let addr = DEMCR::address() as usize as u32;
        let mut w = demcr::W::from(demcr::R::from(self.memory_read_word(addr)?));
        w.VC_CORERESET(if halt { 1 } else { 0 });
        self.memory_write_word(addr, w.into())?;

        let addr = AIRCR::address() as usize as u32;
        let mut w = aircr::W::from(aircr::R::from(self.memory_read_word(addr)?));
        w.VECTKEY(VECTKEY).SYSRESETREQ(1);
        self.memory_write_word(addr, w.into())?;

        // reset causes AHB_AP information to be lost; invalidate the cached state
        self.ap_bank = None;
        self.banked_data_mode = false;
        self.tar = None;
        self.debugen = Some(true);

        // wait for system and debug power-up
        let mut stat = self.read_register(adiv5::Register::DP_STAT)?;
        while stat & adiv5::DP_STAT_CDBGPWRUPACK == 0 {
            self.brief_sleep();
            stat = self.read_register(adiv5::Register::DP_STAT)?;
        }

        while stat & adiv5::DP_STAT_CSYSPWRUPACK == 0 {
            self.brief_sleep();
            stat = self.read_register(adiv5::Register::DP_STAT)?;
        }

        info!("... target reset");
        Ok(())
    }

    /// [ARM Cortex-M] Enables halting debug (DHCSR.C_DEBUGEN <- 1)
    ///
    /// Modifying some Cortex-M registers requires halting debug to be enabled
    pub fn set_debugen(&mut self) -> Result<(), anyhow::Error> {
        if self.debugen == Some(true) {
            return Ok(());
        }

        let addr = DHCSR::address() as usize as u32;
        let dhcsr = dhcsr::R::from(self.memory_read_word(addr)?);
        if dhcsr.C_DEBUGEN() == 0 {
            info!("enabling halting debug mode");
            let mut w = dhcsr::W::from(dhcsr);
            w.DBGKEY(DBGKEY).C_DEBUGEN(1);
            self.memory_write_word(addr, w.into())?;
            let dhcsr = dhcsr::R::from(self.memory_read_word(addr)?);

            if dhcsr.C_DEBUGEN() == 0 {
                bail!(
                    "failed to enable halting debug mode (DHCSR = {:#010x})",
                    u32::from(dhcsr)
                );
            }
        }

        self.debugen = Some(true);

        Ok(())
    }

    /// [ARM Cortex-M] Writes `val` to the specified target's core `reg`-ister
    pub fn write_core_register(&mut self, reg: Register, val: u32) -> Result<(), anyhow::Error> {
        const WRITE: u8 = 1;

        if !self.is_halted()? {
            bail!("core must be halted before writing to a core register")
        }

        info!("writing Cortex-M register {:?} ... ", reg);

        let word = self.memory_write_word(DCRDR::address() as u32, val)?;

        let mut w = dcrsr::W::zero();
        w.REGWnR(WRITE).REGSEL(reg.regsel());
        self.memory_write_word(DCRSR::address() as u32, w.into())?;

        let addr = DHCSR::address() as u32;
        loop {
            let dhcsr = dhcsr::R::from(self.memory_read_word(addr)?);
            if dhcsr.S_REGRDY() != 0 {
                break;
            }
            self.brief_sleep();
        }

        info!("{:?} <- {:#010x}", reg, val);

        Ok(word)
    }
}
