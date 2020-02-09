//! API to access a DAP device using the HID interface
//!
//! # References
//!
//! - 'ADIv5': ARM Debug Interface Architecture Specification ADIv5.0 to ADIv5.2 (ARM IHI 0031C)
//! - 'ARMV7-M ARM': ARMv7-M Architecture Reference Manual (ARM DDI 0403E.d)
//! - 'CoreSight': CoreSight Components Technical Reference Manual (ARM DDI 0314H)
//! - 'Cortex-M4 TRM': Cortex-M4 Technical Reference Manual (ARM DDI 0439B)
//! - [CMSIS-DAP 2.0](https://arm-software.github.io/CMSIS_5/DAP/html/index.html)

#![deny(warnings)]

use core::time::Duration;
use std::thread;

use anyhow::anyhow;
use hidapi::{HidApi, HidDevice};
use log::{debug, info};

// comment indicates the abstraction level (0: lowest, 9: highest)
pub mod adiv5; // 2
mod ahb_ap; // 3
pub mod cortex_m; // 4
pub mod dap; // 1
mod hid; // 0
mod sealed;
mod util;

/// A CMSIS-DAP Debug Unit
pub struct Dap {
    device: HidDevice,
    buffer: Box<[u8]>,

    // property of the target
    packet_size: u16,

    // transfer buffer
    total_requests: u8,
    read_requests: u8,
    cursor: u16,
    ap_bank: Option<adiv5::ApBank>,
    banked_data_mode: bool,

    // AHB-AP specific
    tar: Option<u32>,

    // Cortex-M specific
    debugen: Option<bool>,
}

/* # Utility functions */
// XXX are these reasonable defaults?
const DEFAULT_SWD_FREQUENCY: u32 = 4_000_000;
const DEFAULT_WAIT_RETRY: u16 = 1;

impl crate::Dap {
    /// Opens the DAP Debug Unit that matches the given vendor and product IDs
    pub fn open(vendor: u16, product: u16) -> Result<Self, anyhow::Error> {
        let hid = HidApi::new()?;
        let device = hid.open(vendor, product)?;

        let mut dap = Self {
            buffer: Box::new([crate::hid::REPORT_ID; 5]),
            device,

            ap_bank: None,
            banked_data_mode: false,
            cursor: 1,
            debugen: None,
            read_requests: 0,
            tar: None,
            total_requests: 0,

            // initial conservative assumption
            packet_size: 4,
        };

        // grow the buffer to match the target's supported packet size
        dap.packet_size = dap.packet_size()?;
        dap.buffer = Box::<[u8]>::from(vec![0; usize::from(dap.packet_size)]);

        Ok(dap)
    }

    /// Configures the Debug Unit to use the SWD interface, puts the target in SWD mode and powers
    /// up the target's debug domain
    pub fn default_swd_configuration(&mut self) -> Result<(), anyhow::Error> {
        info!("confirming SWD support");
        let caps = self.capabilities()?;
        if caps & dap::CAPABILITIES_SWD == 0 {
            return Err(anyhow!("DAP does not support SWD"));
        }

        info!("initializing SWD interface");
        self.connect(dap::Mode::SWD)?;

        info!(
            "setting SWD clock frequency to {} MHz",
            DEFAULT_SWD_FREQUENCY / 1_000_000
        );
        self.swj_clock(DEFAULT_SWD_FREQUENCY)?;

        info!(
            "configuring transfer to retry on WAIT responses from the target"
        );
        self.transfer_configure(0, DEFAULT_WAIT_RETRY, 0)?;

        info!("switching the target's connection mode from JTAG to SWD");
        self.swj_sequence(dap::JTAG_TO_SWD_SWJ_SEQUENCE)?;

        // XXX for some reason debug power-up fails without first reading DPIDR
        let dpidr = self.read_register(adiv5::Register::DP_DPIDR)?;
        if (dpidr & ((1 << 12) - 1))
            != (adiv5::DP_DPIDR_RESERVED | adiv5::DP_DPIDR_MANUFACTURER_ARM)
        {
            return Err(anyhow!(
                "target device doesn't appear to be an ARM device (DPIDR = {:#010x})",
                dpidr
            ));
        }
        let version = (dpidr >> 12) & ((1 << 3) - 1);
        info!(
            "Debug Port version: {} (DPIDR = {:#010x})",
            if version == 2 {
                "DPv2"
            } else if version == 1 {
                "DPv1"
            } else {
                return Err(anyhow!("only DPv1 and DPv2 are supported"));
            },
            dpidr
        );

        // // "Tools can only initiate an AP transfer when CDBGPWRUPREQ and
        // // CDBGPWRUPACK are asserted HIGH. If CDBGPWRUPREQ or CDBGPWRUPACK is
        // // LOW, any AP transfer generates an immediate fault response.", section
        // // 2.4.2 of ADIv5
        let stat = self.read_register(adiv5::Register::DP_STAT)?;
        let stat = if stat & adiv5::DP_STAT_CDBGPWRUPACK == 0 {
            debug!("debug power-up request");
            self.push_dap_transfer_request(
                adiv5::Register::DP_CTRL,
                dap::Request::Write(
                    (stat & adiv5::DP_CTRL_CSYSPWRUPREQ)
                        | adiv5::DP_CTRL_CDBGPWRUPREQ,
                ),
            );
            self.push_dap_transfer_request(
                adiv5::Register::DP_STAT,
                dap::Request::Read,
            );
            let stat = self.execute_dap_transfer()?[0];
            if stat & adiv5::DP_STAT_CDBGPWRUPACK == 0 {
                return Err(anyhow!("debug power-up request failed"));
            }
            stat
        } else {
            stat
        };

        if stat & adiv5::DP_STAT_CSYSPWRUPACK == 0 {
            debug!("system power-up request");
            self.push_dap_transfer_request(
                adiv5::Register::DP_CTRL,
                dap::Request::Write(
                    (stat & adiv5::DP_CTRL_CDBGPWRUPREQ)
                        | adiv5::DP_CTRL_CSYSPWRUPREQ,
                ),
            );
            self.push_dap_transfer_request(
                adiv5::Register::DP_STAT,
                dap::Request::Read,
            );
            let stat = self.execute_dap_transfer()?[0];
            if stat & adiv5::DP_STAT_CSYSPWRUPACK == 0 {
                return Err(anyhow!("system power-up request failed"));
            }
        }

        Ok(())
    }

    /// Sleep for a bit to let the target make progress
    fn brief_sleep(&self) {
        thread::sleep(Duration::from_micros(
            64 * 1_000_000 / u64::from(DEFAULT_SWD_FREQUENCY),
        ));
    }
}
