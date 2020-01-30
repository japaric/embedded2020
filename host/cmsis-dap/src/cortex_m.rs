//! Cortex-M specific operations

use core::fmt;

use anyhow::anyhow;
use log::info;

use crate::adiv5;

/// Cortex-M register that the DAP can read
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Register {
    /// Stack Pointer
    SP = 0b1101,

    /// Program Counter (AKA Debug Return Address)
    PC = 0b1111,
}

impl Register {
    fn regsel(&self) -> u32 {
        *self as u32
    }
}

impl crate::Dap {
    /// Halts a Cortex-M target
    pub fn cortex_m_halt(&mut self) -> Result<(), anyhow::Error> {
        info!("halting the target ...");

        self.cortex_m_set_debugen()?;

        let addr = DHCSR;
        let val = self.memory_read_word(addr)?;
        if val & DHCSR_C_HALT == 0 {
            self.memory_write_word(addr, DHCSR_DBGKEY | DHCSR_C_HALT | DHCSR_C_DEBUGEN)?;
            let val = self.memory_read_word(addr)?;

            if val & DHCSR_C_HALT == 0 {
                return Err(anyhow!("failed to halt the target (DHCSR = {:#010x})", val));
            }
        }

        info!("... target halted");

        Ok(())
    }

    /// Reads the specified target's core register
    pub fn read_cortex_m_register(&mut self, reg: Register) -> Result<u32, anyhow::Error> {
        info!("reading Cortex-M register {:?} ... ", reg);

        self.memory_write_word(DCRSR, DCRSR_REGWNR_READ | reg.regsel())?;

        while self.memory_read_word(DHCSR)? & DHCSR_S_REGRDY == 0 {
            self.brief_sleep();
        }

        let word = self.memory_read_word(DCRDR)?;

        info!("{:?} -> {:#010x}", reg, word);

        Ok(word)
    }

    /// Requests a system reset
    pub fn cortex_m_sysresetreq(&mut self) -> Result<(), anyhow::Error> {
        info!("resetting the target with SYSRESETREQ ...");

        self.cortex_m_set_debugen()?;

        let addr = AIRCR;
        let val = self.memory_read_word(addr)?;
        let ro_mask = (u32::from(u16::max_value()) << 16) | (1 << 15) | (0b111 << 8);
        let new_val = (val & !ro_mask) | AIRCR_VECTKEY | AIRCR_SYSRESETREQ;
        self.memory_write_word(addr, new_val)?;

        // reset causes AHB_AP information to be lost; invalidate the cached state
        self.ap_bank = None;
        self.banked_data_mode = false;
        self.tar = None;
        // NOTE it is possible to preserve C_DEBUGEN by catching the reset vector (see DEMCR)
        self.debugen = Some(false);

        // wait for system and debug power-up
        let mut stat = self.read_adiv5_register(adiv5::Register::DP_STAT)?;
        while stat & adiv5::DP_STAT_CDBGPWRUPACK == 0 {
            self.brief_sleep();
            stat = self.read_adiv5_register(adiv5::Register::DP_STAT)?;
        }

        while stat & adiv5::DP_STAT_CSYSPWRUPACK == 0 {
            self.brief_sleep();
            stat = self.read_adiv5_register(adiv5::Register::DP_STAT)?;
        }

        info!("... target reset");
        Ok(())
    }

    /// Enables halting debug (DHCSR.C_DEBUGEN <- 1)
    ///
    /// Modifying some Cortex-M registers requires halting debug to be enabled
    fn cortex_m_set_debugen(&mut self) -> Result<(), anyhow::Error> {
        if self.debugen == Some(true) {
            return Ok(());
        }

        let addr = DHCSR;
        let val = self.memory_read_word(addr)?;
        if val & DHCSR_C_DEBUGEN == 0 {
            info!("enabling halting debug mode");
            self.memory_write_word(addr, DHCSR_DBGKEY | DHCSR_C_DEBUGEN)?;
            let val = self.memory_read_word(addr)?;

            if val & DHCSR_C_DEBUGEN == 0 {
                return Err(anyhow!(
                    "failed to enable halting debug mode (DHCSR = {:#010x})",
                    val
                ));
            }
        }

        self.debugen = Some(true);

        Ok(())
    }
}

/* # MMIO Registers */

/* ## CPUID */

// section B3.2.3 of ARMv7-M TRM
/// CPUID register
pub const CPUID: u32 = 0xe000_ed00;

/* ## DHCSR */

/// Debug Halting Control and Status Register
pub const DHCSR: u32 = 0xe000_edf0;

/// Debug Key
pub const DHCSR_DBGKEY: u32 = 0xA05F << 16;

/// A handshake flag for transfers through the DCRDR
pub const DHCSR_S_REGRDY: u32 = 1 << 16;

/// Processor halt bit
pub const DHCSR_C_HALT: u32 = 1 << 1;

/// Halting debug enable bit
pub const DHCSR_C_DEBUGEN: u32 = 1 << 0;

/* ## DCRSR */

/// Debug Core Register Selector Register
pub const DCRSR: u32 = 0xe000_edf4;

/// Access type for the transfer: Read
pub const DCRSR_REGWNR_READ: u32 = 0 << 16;

/// Access type for the transfer: Write
pub const DCRSR_REGWNR_WRITE: u32 = 1 << 16;

/* ## DCRDR */

/// Debug Core Register Data Register
pub const DCRDR: u32 = 0xe000_edf8;

/* ## AIRCR */

/// Application Interrupt and Reset Control Register
pub const AIRCR: u32 = 0xe000_ed0c;

/// Vector Key
pub const AIRCR_VECTKEY: u32 = 0x05fa << 16;

/// System Reset Request
pub const AIRCR_SYSRESETREQ: u32 = 1 << 2;

/// CPUID register
pub struct Cpuid {
    implementer: u8,
    variant: u8,
    constant: u8,
    partno: u16,
    revision: u8,
}

impl From<u32> for Cpuid {
    fn from(word: u32) -> Self {
        let implementer = (word >> 24) as u8;
        let variant = ((word >> 20) & ((1 << 4) - 1)) as u8;
        let constant = ((word >> 16) & ((1 << 4) - 1)) as u8;
        let partno = ((word >> 4) & ((1 << 12) - 1)) as u16;
        let revision = (word & ((1 << 4) - 1)) as u8;

        Self {
            implementer,
            constant,
            variant,
            partno,
            revision,
        }
    }
}

impl Cpuid {
    /// Returns the bits of the CPUID
    pub fn bits(&self) -> u32 {
        u32::from(self.implementer) << 24
            | u32::from(self.variant) << 20
            | u32::from(self.constant) << 16
            | u32::from(self.partno) << 4
            | u32::from(self.revision)
    }

    /// Checks if the implementer is set to ARM
    pub fn implementer_is_arm(&self) -> bool {
        const CPUID_IMPLEMENTER_ARM: u8 = 0x41;

        self.implementer == CPUID_IMPLEMENTER_ARM
    }

    /// Returns the part number field
    pub fn partno(&self) -> Partno {
        const CPUID_CONSTANT_ARMV6M: u8 = 0xc;
        const CPUID_CONSTANT_ARMV7M: u8 = 0xf;

        if self.implementer_is_arm() {
            if self.constant == CPUID_CONSTANT_ARMV6M {
                // or v8-M baseline
                if self.partno == 0xc20 {
                    Partno::CortexM0
                } else if self.partno == 0xc60 {
                    Partno::CortexM0Plus
                } else if self.partno == 0xd20 {
                    Partno::CortexM23
                } else {
                    Partno::Unknown
                }
            } else if self.constant == CPUID_CONSTANT_ARMV7M {
                // or v8-M mainline
                if self.partno == 0xc23 {
                    Partno::CortexM3
                } else if self.partno == 0xC24 {
                    Partno::CortexM4
                } else if self.partno == 0xc27 {
                    Partno::CortexM7
                } else if self.partno == 0xd21 {
                    Partno::CortexM33
                } else {
                    Partno::Unknown
                }
            } else {
                Partno::Unknown
            }
        } else {
            Partno::Unknown
        }
    }
}

/// Part number
pub enum Partno {
    /// ARM Cortex-M0
    CortexM0,

    /// ARM Cortex-M0+
    CortexM0Plus,

    /// ARM Cortex-M3
    CortexM3,

    /// ARM Cortex-M4
    CortexM4,

    /// ARM Cortex-M7
    CortexM7,

    /// ARM Cortex-M23
    CortexM23,

    /// ARM Cortex-M33
    CortexM33,

    /// Unknown part
    Unknown,
}

impl fmt::Display for Partno {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Partno::CortexM0 => "ARM Cortex-M0",
            Partno::CortexM0Plus => "ARM Cortex-M0+",
            Partno::CortexM3 => "ARM Cortex-M3",
            Partno::CortexM4 => "ARM Cortex-M4",
            Partno::CortexM7 => "ARM Cortex-M7",
            Partno::CortexM23 => "ARM Cortex-M23",
            Partno::CortexM33 => "ARM Cortex-M33",
            Partno::Unknown => "unknown part",
        };

        f.write_str(s)
    }
}
