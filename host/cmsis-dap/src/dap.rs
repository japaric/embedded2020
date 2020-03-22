//! CMSIS-DAP operations

use core::{cmp::Ordering, fmt};

use anyhow::anyhow;
use arrayref::array_ref;
use log::debug;

use crate::{adiv5, sealed::Data as _};

const LEN_BYTE: u8 = 1;
const LEN_SHORT: u8 = 2;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum Command {
    DAP_Info = 0x00,
    DAP_Connect = 0x02,

    DAP_TransferConfigure = 0x04,
    DAP_Transfer = 0x05,
    DAP_TransferBlock = 0x06,

    DAP_SWJ_Clock = 0x11,
    DAP_SWJ_Sequence = 0x12,
}

impl crate::hid::AsLeBytes for Command {
    fn as_le_bytes(&self, f: impl FnOnce(&[u8])) {
        f(&[*self as u8])
    }
}

impl PartialEq<Command> for u8 {
    fn eq(&self, cmd: &Command) -> bool {
        *self == *cmd as u8
    }
}

/* # General commands */

/* ## DAP_Info */

const DAP_INFO_CAPABILITIES: u8 = 0xf0;
pub(crate) const CAPABILITIES_SWD: u8 = 1;

// const DAP_INFO_PACKET_COUNT: u8 = 0xfe;

const DAP_INFO_PACKET_SIZE: u8 = 0xff;

impl crate::Dap {
    // FIXME this may return `LEN_SHORT`
    /// Returns the DAP Debug Unit capabilities
    pub fn capabilities(&mut self) -> Result<u8, anyhow::Error> {
        const CMD: Command = Command::DAP_Info;

        debug!("{:?} Capabilities ...", CMD);

        self.hid_push(CMD);
        self.hid_push(DAP_INFO_CAPABILITIES);
        self.hid_flush()?;

        // CMD_DAP_INFO: u8 - LEN_BYTE: u8 - DATA: u8
        let resp = self.hid_read(3)?;
        if resp[0] != CMD || resp[1] != LEN_BYTE {
            return Err(anyhow!("`DAP_Info Capabilities` failed"));
        }
        let caps = resp[2];

        debug!("... {:#02x}", caps);

        Ok(caps)
    }

    /// Returns the DAP Debug Unit maximum packet size
    pub fn packet_size(&mut self) -> Result<u16, anyhow::Error> {
        const CMD: Command = Command::DAP_Info;

        debug!("`{:?} PacketSize` ...", CMD);

        self.hid_push(CMD);
        self.hid_push(DAP_INFO_PACKET_SIZE);
        self.hid_flush()?;

        // CMD_DAP_INFO: u8 - LEN_SHORT: u8 - PacketSize: u16
        let resp = self.hid_read(4)?;
        if resp[0] != CMD || resp[1] != LEN_SHORT {
            return Err(anyhow!("`DAP_Info PacketSize` failed"));
        }
        let packet_size = u16::from_le_bytes(*array_ref!(resp, 2, 2));

        debug!("... {} bytes", packet_size);

        Ok(packet_size)
    }
}

/* ## DAP_Connect */
// const DAP_CONNECT_DEFAULT: u8 = 0;
const DAP_CONNECT_SWD: u8 = 1;
// const DAP_CONNECT_JTAG: u8 = 2;

/// DAP communication modes
pub enum Mode {
    /// Serial-Wire-Debug
    SWD,
}

impl crate::Dap {
    /// Connects to the target using the specified `mode`
    pub fn connect(&mut self, _mode: Mode) -> Result<(), anyhow::Error> {
        const CMD: Command = Command::DAP_Connect;

        debug!("{:?} SWD", CMD);

        self.hid_push(CMD);
        self.hid_push(DAP_CONNECT_SWD);
        self.hid_flush()?;

        // CMD_DAP_CONNECT: u8 - DAP_CONNECT_SWD: u8
        let resp = self.hid_read(2)?;
        if resp[0] != CMD || resp[1] != DAP_CONNECT_SWD {
            return Err(anyhow!("`{:?}` failed", CMD));
        }

        Ok(())
    }
}

/* # Common SWD/JTAG commands */

/* ## DAP_SWJ_Clock */

impl crate::Dap {
    /// Sets the clock `frequency` for the JTAG and SWD communication mode
    pub fn swj_clock(&mut self, frequency: u32) -> Result<(), anyhow::Error> {
        const CMD: Command = Command::DAP_SWJ_Clock;

        debug!("{:?} {}", CMD, frequency);
        self.hid_push(CMD);
        self.hid_push(frequency);
        self.hid_flush()?;
        self.check_response(CMD)
    }
}

/* ## DAP_SWJ_Sequenc */

/// SWJ sequence to switch from JTAG mode to SWD mode
pub static JTAG_TO_SWD_SWJ_SEQUENCE: &[u8] = &[
    // at least 50 cycles of SWDIO/TMS high
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // 16-bit JTAG-to-SWD select sequence
    0x9e, 0xe7, //
    // at least 50 cycles of SWDIO/TMS high
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    // at least 2 idle cycles (section 4.4.3 of ADIv5)
    0x00,
];

impl crate::Dap {
    /// Generates a SWJ sequence (SWDIO/TMS = bit banging pin & SWCLK/TCK = clock pin)
    pub fn swj_sequence(&mut self, data: &[u8]) -> Result<(), anyhow::Error> {
        const CMD: Command = Command::DAP_SWJ_Sequence;

        let count = data
            .len()
            .checked_mul(8)
            .and_then(|bits| match bits.cmp(&256) {
                Ordering::Equal => Some(0),
                Ordering::Greater => None,
                Ordering::Less => Some(bits as u8),
            })
            .expect("sequence is longer than 256 bits");
        debug!("DAP_SWJ_Sequence <{} bits>", count);
        self.hid_push(CMD);
        self.hid_push(count);
        self.hid_push(data);
        self.hid_flush()?;
        self.check_response(CMD)?;
        Ok(())
    }
}

/* # Transfer commands */

/* ## DAP_TransferConfigure */

impl crate::Dap {
    /// Sets parameters for the `DAP_Transfer` and `DAP_TransferBlock` operations
    pub fn transfer_configure(
        &mut self,
        idle_cycles: u8,
        wait_retry: u16,
        match_retry: u16,
    ) -> Result<(), anyhow::Error> {
        const CMD: Command = Command::DAP_TransferConfigure;

        debug!(
            "{:?}(idle_cycles = {}, wait_retry = {}, match_retry = {})",
            CMD, idle_cycles, wait_retry, match_retry,
        );
        self.hid_push(CMD);
        self.hid_push(idle_cycles);
        self.hid_push(wait_retry);
        self.hid_push(match_retry);
        self.hid_flush()?;
        self.check_response(CMD)
    }
}

/* ## DAP_Transfer */

const TRANSFER_RNW_WRITE: u8 = 0 << 1;
const TRANSFER_RNW_READ: u8 = 1 << 1;
const TRANSFER_ACK_OK: u8 = 1;
const TRANSFER_DAP_INDEX: u8 = 0; // always 0 for SWD interfaces

/// Requested access
#[derive(Clone, Copy)]
pub enum Request {
    /// Read access
    Read,

    /// Write access
    Write(u32),
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Request::Read => f.write_str("Read"),
            Request::Write(val) => write!(f, "Write({:#010x})", val),
        }
    }
}

impl adiv5::Register {
    fn request(self) -> u8 {
        use adiv5::Register::*;

        const REQUEST_DP: u8 = 0;
        const REQUEST_AP: u8 = 1;

        match self {
            DP_DPIDR => REQUEST_DP,
            DP_CTRL => 0x4 | REQUEST_DP,
            DP_STAT => 0x4 | REQUEST_DP,
            DP_SELECT => 0x8 | REQUEST_DP,
            DP_RDBUFF => 0xc | REQUEST_DP,

            AHB_AP_CSW => REQUEST_AP,
            AHB_AP_TAR => 0x4 | REQUEST_AP,
            AHB_AP_DRW => 0xc | REQUEST_AP,

            AHB_AP_BD0 => REQUEST_AP,
            AHB_AP_BD1 => 0x4 | REQUEST_AP,
            AHB_AP_BD2 => 0x8 | REQUEST_AP,
            AHB_AP_BD3 => 0xc | REQUEST_AP,
        }
    }
}

impl crate::Dap {
    /// Pushes a DAP transfer request into the internal buffer
    pub fn push_dap_transfer_request(&mut self, reg: adiv5::Register, req: Request) {
        const CMD: Command = Command::DAP_Transfer;

        // change bank, if required
        if let Some(bank) = reg.ap_bank() {
            if self.ap_bank.is_none() || self.ap_bank != Some(bank) {
                match bank {
                    adiv5::ApBank::AHB_AP(n) => {
                        self.push_dap_transfer_request(
                            adiv5::Register::DP_SELECT,
                            Request::Write(
                                adiv5::DP_SELECT_APSEL_AHB_AP
                                    | (u32::from(n) << adiv5::DP_SELECT_APBANKSEL_OFFSET),
                            ),
                        );

                        self.ap_bank = Some(bank);
                    }
                }
            }
        }

        debug!("{:?} += {:?} @ {:?}", CMD, req, reg);

        let count = self.total_requests + 1;
        if self.total_requests == 0 {
            assert_eq!(self.cursor, 1);

            // add header
            self.hid_push(CMD);
            self.hid_push(TRANSFER_DAP_INDEX);
            self.hid_push(count);
        } else {
            self.hid_rewrite(3, count);
        }
        self.total_requests += 1;

        if let Request::Write(val) = req {
            self.hid_push(reg.request() | TRANSFER_RNW_WRITE);
            self.hid_push(val);
        } else {
            self.hid_push(reg.request() | TRANSFER_RNW_READ);
            self.read_requests += 1;
        }
    }

    /// Executes all standing DAP transfer requests in a single `DAP_Transfer` operation
    ///
    /// # Panics
    ///
    /// This method panics if there are no outstanding DAP transfer requests
    pub fn execute_dap_transfer(&mut self) -> Result<Vec<u32>, anyhow::Error> {
        const CMD: Command = Command::DAP_Transfer;
        /// Response Header Size
        const RHS: u16 = 3;

        assert_ne!(self.total_requests, 0, "no transfer request was enqueued");

        debug!(
            "{:?} <{} request{}>",
            CMD,
            self.total_requests,
            if self.total_requests != 1 { "s" } else { "" }
        );

        self.hid_flush()?;

        let total_requests = self.total_requests;
        let read_requests = self.read_requests;
        self.total_requests = 0;
        self.read_requests = 0;

        let resp = self.hid_read(RHS + 4 * u16::from(read_requests))?;

        if resp[0] != CMD || resp[1] != total_requests || resp[2] != TRANSFER_ACK_OK {
            return Err(anyhow!("`{:?}` failed (resp = {:?})", CMD, resp));
        }

        let words = resp[usize::from(RHS)..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(*array_ref!(chunk, 0, 4)))
            .collect();
        for word in &words {
            debug!("DAP_Transfer -> {:#010x}", word);
        }
        Ok(words)
    }
}

/* ## DAP_TransferBlock */
impl crate::Dap {
    /// Reads a block of data from a single register
    pub fn transfer_block_read(
        &mut self,
        reg: adiv5::Register,
        count: u16,
    ) -> Result<&[u8], anyhow::Error> {
        const CMD: Command = Command::DAP_TransferBlock;
        /// Response Header Size
        const RHS: u16 = 4;

        debug!("DAP_TransferBlock R {:?} {}", reg, count);

        // add header
        self.hid_push(CMD);
        self.hid_push(TRANSFER_DAP_INDEX);
        self.hid_push(count);
        self.hid_push(reg.request() | TRANSFER_RNW_READ);

        self.hid_flush()?;

        let resp = self.hid_read(RHS + count * u32::BYTES)?;

        if resp[0] != CMD || resp[1..3] != count.to_le_bytes()[..] || resp[3] != TRANSFER_ACK_OK {
            return Err(anyhow!("`{:?}` failed", CMD));
        }

        Ok(&resp[usize::from(RHS)..])
    }

    /// Writes a block of data into a single register
    pub fn transfer_block_write(
        &mut self,
        reg: adiv5::Register,
        data: &[u32],
    ) -> Result<(), anyhow::Error> {
        const CMD: Command = Command::DAP_TransferBlock;
        /// Response Header Size
        const RHS: u16 = 4;

        let count = data.len();
        assert!(count < usize::from(u16::max_value()));
        let count = count as u16;

        debug!("DAP_TransferBlock W {:?} {}", reg, count);

        // add header
        self.hid_push(CMD);
        self.hid_push(TRANSFER_DAP_INDEX);
        self.hid_push(count);
        self.hid_push(reg.request() | TRANSFER_RNW_WRITE);
        for word in data {
            self.hid_push(word);
        }

        self.hid_flush()?;

        let resp = self.hid_read(RHS)?;

        if resp[0] != CMD || resp[1..3] != count.to_le_bytes()[..] || resp[3] != TRANSFER_ACK_OK {
            return Err(anyhow!("`{:?}` failed", CMD));
        }

        Ok(())
    }
}

/* # Response Status */
const DAP_OK: u8 = 0;

impl crate::Dap {
    fn check_response(&mut self, cmd: Command) -> Result<(), anyhow::Error> {
        let resp = self.hid_read(2)?;

        if resp[0] == cmd && resp[1] == DAP_OK {
            Ok(())
        } else {
            Err(anyhow!("`{:?}` failed", cmd))
        }
    }
}
