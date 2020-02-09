//! Cortex-M specific operations

use anyhow::anyhow;
use cm::{
    dcb::{dcrsr, dhcsr, DCRDR, DCRSR, DHCSR},
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
    /// [Cortex-M] Halts a Cortex-M target
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
                return Err(anyhow!(
                    "failed to halt the target (DHCSR = {:#010x})",
                    u32::from(dhcsr)
                ));
            }
        }

        info!("... target halted");

        Ok(())
    }

    /// Checks if the target is currently halted
    pub fn is_halted(&mut self) -> Result<bool, anyhow::Error> {
        Ok(if self.debugen != Some(false) {
            false
        } else {
            dhcsr::R::from(
                self.memory_read_word(DHCSR::address() as usize as u32)?,
            )
            .C_HALT()
                != 0
        })
    }

    /// [ARM Cortex-M] Reads the specified target's core register
    pub fn read_core_register(
        &mut self,
        reg: Register,
    ) -> Result<u32, anyhow::Error> {
        const READ: u8 = 0;

        info!("reading Cortex-M register {:?} ... ", reg);

        let mut w = dcrsr::W::zero();
        w.REGWnR(READ).REGSEL(reg.regsel());
        self.memory_write_word(DCRSR::address() as usize as u32, w.into())?;

        let dhcsr = dhcsr::R::from(
            self.memory_read_word(DHCSR::address() as usize as u32)?,
        );
        while dhcsr.S_REGRDY() == 0 {
            self.brief_sleep();
        }

        let word = self.memory_read_word(DCRDR::address() as usize as u32)?;

        info!("{:?} -> {:#010x}", reg, word);

        Ok(word)
    }

    /// [ARM Cortex-M] Requests a system reset
    pub fn sysresetreq(&mut self) -> Result<(), anyhow::Error> {
        /// Vector Key
        const VECTKEY: u16 = 0x05fa;

        info!("resetting the target with SYSRESETREQ ...");

        self.set_debugen()?;

        let addr = AIRCR::address() as usize as u32;
        let mut w =
            aircr::W::from(aircr::R::from(self.memory_read_word(addr)?));
        // let ro_mask =
        // (u32::from(u16::max_value()) << 16) | (1 << 15) | (0b111 << 8);
        w.VECTKEY(VECTKEY).SYSRESETREQ(1);
        // let new_val = (val & !ro_mask) | AIRCR_VECTKEY | AIRCR_SYSRESETREQ;
        self.memory_write_word(addr, w.into())?;

        // reset causes AHB_AP information to be lost; invalidate the cached state
        self.ap_bank = None;
        self.banked_data_mode = false;
        self.tar = None;
        // NOTE it is possible to preserve C_DEBUGEN by catching the reset vector (see DEMCR)
        self.debugen = Some(false);

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
    fn set_debugen(&mut self) -> Result<(), anyhow::Error> {
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
                return Err(anyhow!(
                    "failed to enable halting debug mode (DHCSR = {:#010x})",
                    u32::from(dhcsr)
                ));
            }
        }

        self.debugen = Some(true);

        Ok(())
    }
}
