//! ARM Debug Interface v5

use crate::dap;

/// ADIv5 registers
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Register {
    /// Debug Port Identification Register
    DP_DPIDR,
    /// DP Control register
    DP_CTRL,

    /// DP Status register
    DP_STAT,

    /// AP Select Register
    DP_SELECT,

    /// Read Buffer
    DP_RDBUFF,

    /// Control/Status Word
    AHB_AP_CSW,

    /// Transfer Address
    AHB_AP_TAR,

    /// Data Read/Write
    AHB_AP_DRW,

    /// Banked Data 0
    AHB_AP_BD0,

    /// Banked Data 1
    AHB_AP_BD1,

    /// Banked Data 2
    AHB_AP_BD2,

    /// Banked Data 3
    AHB_AP_BD3,
}

impl Register {
    pub(crate) fn banked_data(addr: u32) -> Self {
        assert_eq!(addr % 4, 0, "address not 4-byte aligned");

        match addr & 0xf {
            0x0 => Register::AHB_AP_BD0,
            0x4 => Register::AHB_AP_BD1,
            0x8 => Register::AHB_AP_BD2,
            0xc => Register::AHB_AP_BD3,
            _ => unreachable!(),
        }
    }

    pub(crate) fn ap_bank(&self) -> Option<ApBank> {
        match *self {
            Register::DP_DPIDR
            | Register::DP_CTRL
            | Register::DP_STAT
            | Register::DP_SELECT
            | Register::DP_RDBUFF => None,

            Register::AHB_AP_CSW | Register::AHB_AP_TAR | Register::AHB_AP_DRW => {
                Some(ApBank::AHB_AP(0))
            }

            Register::AHB_AP_BD0
            | Register::AHB_AP_BD1
            | Register::AHB_AP_BD2
            | Register::AHB_AP_BD3 => Some(ApBank::AHB_AP(1)),
        }
    }
}

/// AP bank
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ApBank {
    AHB_AP(u8),
}

impl crate::Dap {
    /// Reads the specified ADIv5 `register`
    ///
    /// # Panics
    ///
    /// This method panics if there are outstanding DAP transfer requests -- `execute_dap_transfer`
    /// must be called before calling this method
    pub fn read_adiv5_register(&mut self, reg: Register) -> Result<u32, anyhow::Error> {
        assert_eq!(self.total_requests, 0, "outstanding DAP transfer requests");
        self.push_dap_transfer_request(reg, dap::Request::Read);
        Ok(self.execute_dap_transfer()?[0])
    }

    /// Writes the given `value` to the specified ADIv5 `register`
    ///
    /// # Panics
    ///
    /// This method panics if there are outstanding DAP transfer requests -- `execute_dap_transfer`
    /// must be called before calling this method
    pub fn write_adiv5_register(&mut self, reg: Register, val: u32) -> Result<(), anyhow::Error> {
        assert_eq!(self.total_requests, 0, "outstanding DAP transfer requests");
        self.push_dap_transfer_request(reg, dap::Request::Write(val));
        self.execute_dap_transfer().map(drop)
    }
}

/* # Register Bit fields */

/* ## DP access port */

/* ### DPIDR register */

/// Reserved field
pub const DP_DPIDR_RESERVED: u32 = 1;

/// JEDEC Manufacturer ID
pub const DP_DPIDR_MANUFACTURER_ARM: u32 = 0x23b << 1;

/* ### STAT / CTRL register */

/// System power-up acknowledge
pub const DP_STAT_CSYSPWRUPACK: u32 = 1 << 31;

/// System power-up request
pub const DP_CTRL_CSYSPWRUPREQ: u32 = 1 << 30;

/// Debug power-up acknowledge
pub const DP_STAT_CDBGPWRUPACK: u32 = 1 << 29;

/// Debug power-up request
pub const DP_CTRL_CDBGPWRUPREQ: u32 = 1 << 28;

/* ### SELECT register */

/// APSEL = AHB-AP
pub const DP_SELECT_APSEL_AHB_AP: u32 = 0x00 << 24;

/// Offset of the APBANKSEL field
pub const DP_SELECT_APBANKSEL_OFFSET: u8 = 4;

/* ## AHB-AP access port */
