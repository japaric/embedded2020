use core::{cell::Cell, cmp};
use std::convert::TryInto as _;

use anyhow::bail;
use arrayref::array_ref;
use log::{debug, info};

use crate::{
    adiv5::{self, ApBank},
    dap::{self, Command},
    sealed::{self, Data as _},
    util,
};

// XXX do other Cortex-M variants have a different "must preserve" value
// see section 8.2.2 of Cortex-M4 TRM (Debug ARM DDI 0439B)
pub(crate) const CSW_MUST_PRESERVE: u32 = (1 << 29) // MasterType
    | (1 << 25) // HPROT1
    | (1 << 24) // Reserved
    | (1 << 6) // DbgStatus
    ;
const CSW_ADDRINC_PACKED: u32 = 0b10 << 4;
const CSW_ADDRINC_BOUNDARY: u32 = 0x400;

impl crate::Dap {
    /// Reads a single word from the target's memory using the AHB-AP (Access Port)
    pub fn memory_read_word(&mut self, addr: u32) -> Result<u32, anyhow::Error> {
        assert_eq!(addr % 4, 0, "address is not 4-byte aligned");

        info!("MRW {:#010x} ...", addr);

        if !self.banked_data_mode {
            self.push_dap_transfer_request(
                adiv5::Register::AHB_AP_CSW,
                dap::Request::Write(CSW_MUST_PRESERVE | u32::CSW_SIZE),
            );
            self.banked_data_mode = true;
        }

        // update the AP_TAR register, if necessary
        let desired_tar = addr & !0xf;
        if self.tar.map(|tar| tar != desired_tar).unwrap_or(true) {
            self.push_dap_transfer_request(
                adiv5::Register::AHB_AP_TAR,
                dap::Request::Write(desired_tar),
            );
            self.tar = Some(desired_tar);
        }

        // use AP_BD* registers
        self.push_dap_transfer_request(adiv5::Register::banked_data(addr), dap::Request::Read);

        let word = self.execute_dap_transfer()?[0];

        info!("... MRW {:#010x} -> {:#010x}", addr, word);

        Ok(word)
    }

    /// Writes a single word into the target's memory using the AHB-AP (Access Port)
    pub fn memory_write_word(&mut self, addr: u32, val: u32) -> Result<(), anyhow::Error> {
        assert_eq!(addr % 4, 0, "address is not 4-byte aligned");

        info!("MWW {:#010x} <- {:#010x}", addr, val);

        if !self.banked_data_mode {
            self.push_dap_transfer_request(
                adiv5::Register::AHB_AP_CSW,
                dap::Request::Write(CSW_MUST_PRESERVE | u32::CSW_SIZE),
            );
            self.banked_data_mode = true;
        }

        // update the AP_TAR register, if necessary
        let desired_tar = addr & !0xf;
        if self.tar.map(|tar| tar != desired_tar).unwrap_or(true) {
            self.push_dap_transfer_request(
                adiv5::Register::AHB_AP_TAR,
                dap::Request::Write(desired_tar),
            );
            self.tar = Some(desired_tar);
        }

        // use AP_BD* registers
        self.push_dap_transfer_request(
            adiv5::Register::banked_data(addr),
            dap::Request::Write(val),
        );

        self.execute_dap_transfer().map(drop)
    }

    /// Reads `n` bytes, half-words or words from the target's memory starting at the specified
    /// `address`
    pub fn memory_read<T>(&mut self, addr: u32, count: u32) -> Result<Vec<T>, anyhow::Error>
    where
        T: sealed::Data,
    {
        assert_eq!(
            addr % u32::from(T::BYTES),
            0,
            "{:#010x} is not {}-byte aligned",
            addr,
            T::BYTES
        );

        if count == 0 {
            return Ok(vec![]);
        }

        info!("MR{} {:#010x} {}", T::acronym(), addr, count);

        self.banked_data_mode = false;

        self.push_dap_transfer_request(
            adiv5::Register::AHB_AP_CSW,
            dap::Request::Write(CSW_MUST_PRESERVE | CSW_ADDRINC_PACKED | T::CSW_SIZE),
        );

        let mut memory = vec![];

        let mut total_bytes = count * u32::from(T::BYTES);
        let mut offset = addr % CSW_ADDRINC_BOUNDARY;
        if offset + total_bytes > CSW_ADDRINC_BOUNDARY {
            // AHB_AP_TAR will need to be updated mid-way to avoid auto-increment wrap-around
            let mut addr = addr;

            while total_bytes != 0 {
                let bytes = cmp::min(total_bytes, CSW_ADDRINC_BOUNDARY - offset);
                offset = 0;
                self.transfer_block_read_csw(addr, bytes / u32::from(T::BYTES), &mut memory)?;

                addr += bytes;
                total_bytes -= bytes;
            }
        } else {
            self.transfer_block_read_csw(addr, count, &mut memory)?;
        }

        Ok(memory)
    }

    /// Reads a half-word and bytes from a circular buffer in a single DAP transaction
    ///
    /// - `hwp` must be a device-side `*const u16` pointer
    /// - `bufp` must be a device-side `*const u8` pointer that points at the beginning of a
    ///   circular buffer
    /// - `cursor` is a cursor (index) into the circular buffer; bytes will be read starting at this
    /// poisition
    /// - `len` is the length of the circular buffer
    // FIXME this duplicates a bunch of code but meh
    pub fn read_hw_and_circbuf(
        &mut self,
        hwp: u32,
        bufp: u32,
        cursor: u16,
        len: u16,
    ) -> Result<(u16, Vec<u8>), anyhow::Error> {
        assert_eq!(hwp % 2, 0);

        const CMD_EC: Command = Command::DAP_ExecuteCommands;
        const RESP_EC: u16 = 2;
        const CMD_T: Command = Command::DAP_Transfer;
        const RESP_T: u16 = 3;
        const CMD_TB: Command = Command::DAP_TransferBlock;
        const RESP_TB: u16 = 4;
        const WORD: u16 = 4;
        // FIXME should not be hardcoded
        const PACKET_SIZE: u16 = 63; // first byte is the HID report number
        let mut ncmds: u8 = 2; // number of command requests

        if !self.supports_atomic_commands()? {
            bail!("`read_hw_and_circbuf` requires support for atomic commands");
        }

        self.banked_data_mode = false;
        self.tar = None; // FIXME cache the right value
        assert_eq!(self.cursor, 1);

        // all operations will be performed in this bank
        let ap_bank = 0;

        let mut resp_len = 0;
        let mut needs_bank_change = false;
        if self.ap_bank.is_none() || self.ap_bank != Some(ApBank::AHB_AP(ap_bank)) {
            self.ap_bank = Some(ApBank::AHB_AP(ap_bank));
            needs_bank_change = true;
        }
        let num_cmd_idx = 2;
        let transfer_count_idx = Cell::new(5);

        self.hid_push(CMD_EC);
        self.hid_push(ncmds); // may be increased (see `num_cmd_idx`)
        resp_len += RESP_EC;

        let requests = Cell::new(0);
        let mut push_dap_transfer_request = |reg: adiv5::Register, req: dap::Request| {
            debug!("[atomic] {:?} += {:?} @ {:?}", CMD_T, req, reg);

            let count = requests.get() + 1;
            if requests.get() == 0 {
                // add header
                self.hid_push(CMD_T);
                self.hid_push(dap::TRANSFER_DAP_INDEX);
                self.hid_push(count);
            } else {
                self.hid_rewrite(transfer_count_idx.get() /* ! */, count);
            }
            requests.set(count);

            if let dap::Request::Write(val) = req {
                self.hid_push(reg.request() | dap::TRANSFER_RNW_WRITE);
                self.hid_push(val);
            } else {
                self.hid_push(reg.request() | dap::TRANSFER_RNW_READ);
            }
        };

        // first command DAP_Transfer
        // change TAR to `hwp`
        requests.set(0);
        if needs_bank_change {
            push_dap_transfer_request(
                adiv5::Register::DP_SELECT,
                dap::Request::Write(
                    adiv5::DP_SELECT_APSEL_AHB_AP
                        | (u32::from(ap_bank) << adiv5::DP_SELECT_APBANKSEL_OFFSET),
                ),
            );
        }
        push_dap_transfer_request(
            adiv5::Register::AHB_AP_CSW,
            dap::Request::Write(CSW_MUST_PRESERVE | u32::CSW_SIZE),
        );
        let tar = adiv5::Register::AHB_AP_TAR;
        push_dap_transfer_request(tar, dap::Request::Write(hwp));

        // read `hwp`
        let drw = adiv5::Register::AHB_AP_DRW;
        push_dap_transfer_request(drw, dap::Request::Read);

        push_dap_transfer_request(
            adiv5::Register::AHB_AP_CSW,
            dap::Request::Write(CSW_MUST_PRESERVE | CSW_ADDRINC_PACKED | u8::CSW_SIZE),
        );

        let start = bufp + u32::from(cursor);
        push_dap_transfer_request(tar, dap::Request::Write(start));
        resp_len += RESP_T + WORD;

        const ACK_OK: u8 = 1;

        // second command DAP_TransferBlock -- see `transfer_block_read`
        self.hid_push(CMD_TB);
        self.hid_push(dap::TRANSFER_DAP_INDEX);
        // FIXME assumes 64B packets
        let mut first_len = 48;
        if cursor + first_len >= len {
            // split transfer
            first_len = len - cursor;
        }

        let count = util::round_up(first_len, 4) / 4;
        self.hid_push(count);
        self.hid_push(drw.request() | dap::TRANSFER_RNW_READ);

        debug!("[atomic] {:?} R {:?} {}", CMD_TB, drw, count);

        resp_len += RESP_TB + 4 * count;

        let second_len = if resp_len + RESP_T + RESP_TB + WORD <= PACKET_SIZE {
            let len = util::round_down(PACKET_SIZE - (resp_len + RESP_T + RESP_TB), 4);
            // third command: DAP_Transfer
            ncmds += 1;
            self.hid_push(CMD_T);
            self.hid_push(dap::TRANSFER_DAP_INDEX);
            self.hid_push(1u8);
            self.hid_push(tar.request() | dap::TRANSFER_RNW_WRITE);
            self.hid_push(bufp);
            resp_len += RESP_T;

            debug!("[atomic] {:?} += Write({:#010x}) @ {:?}", CMD_T, bufp, tar);

            // fourth command: DAP_TransferBlock
            ncmds += 1;
            let count = util::round_up(len, 4) / 4;
            self.hid_push(CMD_TB);
            self.hid_push(dap::TRANSFER_DAP_INDEX);
            self.hid_push(count);
            self.hid_push(drw.request() | dap::TRANSFER_RNW_READ);
            resp_len += RESP_TB + 4 * count;

            debug!("[atomic] {:?} R {:?} {}", CMD_TB, drw, count);

            self.hid_rewrite(num_cmd_idx, ncmds);

            Some(len)
        } else {
            None
        };

        self.hid_flush()?;

        let resp = self.hid_read(resp_len)?;

        if resp[0] == CMD_EC
            && resp[1] == ncmds
            && resp[2] == CMD_T
            && resp[3] == requests.get()
            && resp[4] == ACK_OK
            && resp[9] == CMD_TB
            && resp[10..12] == (util::round_up(first_len, 4) / 4).to_le_bytes()
            && resp[12] == ACK_OK
        {
            let word = &resp[5..9];
            let hw = if hwp % 4 == 0 {
                u16::from_le_bytes([word[0], word[1]])
            } else {
                u16::from_le_bytes([word[2], word[3]])
            };

            let mut bytes = vec![];

            for chunk in resp[13..13 + util::round_up(first_len, 4) as usize].chunks_exact(4) {
                let offset = (start % 4) as usize;
                bytes.extend_from_slice(&chunk[offset..]);
                bytes.extend_from_slice(&chunk[..offset]);
            }

            bytes.truncate(first_len as usize);

            if let Some(len) = second_len {
                // should be multiple of 4 due to the previous `round_down`
                assert_eq!(len % 4, 0);

                let start = 13 + util::round_up(first_len, 4) as usize;

                if resp[start] == CMD_T
                    && resp[start + 1] == 1
                    && resp[start + 2] == ACK_OK
                    && resp[start + 3] == CMD_TB
                    && resp[start + 4..start + 6] == (len / 4).to_le_bytes()
                    && resp[start + 6] == ACK_OK
                {
                    // should be multiple of 4 due to the previous `round_down`
                    assert_eq!(resp[start + 7..].len(), len as usize);

                    for chunk in resp[start + 7..].chunks_exact(4) {
                        let offset = (bufp % 4) as usize;
                        bytes.extend_from_slice(&chunk[offset..]);
                        bytes.extend_from_slice(&chunk[..offset]);
                    }
                } else {
                    bail!("`{:?}` failed", CMD_EC)
                }
            }

            Ok((hw, bytes))
        } else {
            bail!("`{:?}` failed", CMD_EC)
        }
    }

    fn transfer_block_read_csw<T>(
        &mut self,
        mut addr: u32,
        mut total_count: u32,
        memory: &mut Vec<T>,
    ) -> Result<(), anyhow::Error>
    where
        T: sealed::Data,
    {
        /// Request Header Size
        const RHS: u16 = 5;

        assert!(
            addr % CSW_ADDRINC_BOUNDARY + total_count * u32::from(T::BYTES) <= CSW_ADDRINC_BOUNDARY
        );

        if self.tar.map(|tar| tar != addr).unwrap_or(true) {
            self.push_dap_transfer_request(adiv5::Register::AHB_AP_TAR, dap::Request::Write(addr));
            self.tar = Some(addr);
        }
        self.execute_dap_transfer()?;

        let max_count_per_transfer =
            util::round_down(self.packet_size - RHS, u32::BYTES) / T::BYTES;

        while total_count != 0 {
            // NOTE(as u16) `max_count_per_transfer` has type `u16`
            let count = cmp::min(total_count, u32::from(max_count_per_transfer)) as u16;
            let payload_bytes = util::round_up(count * T::BYTES, u32::BYTES);

            let data =
                self.transfer_block_read(adiv5::Register::AHB_AP_DRW, payload_bytes / u32::BYTES)?;

            T::push(memory, data, addr % 4, count);

            total_count -= u32::from(count);
            addr += u32::from(payload_bytes);
        }

        if addr % CSW_ADDRINC_BOUNDARY == 0 {
            self.tar = Some(addr - CSW_ADDRINC_BOUNDARY);
        } else {
            self.tar = Some(addr);
        }

        Ok(())
    }

    /// Writes `bytes` into the target's memory starting at the specified `address`
    pub fn memory_write(&mut self, addr: u32, mut bytes: &[u8]) -> Result<(), anyhow::Error> {
        assert_eq!(
            addr % u32::from(u32::BYTES),
            0,
            "{:#010x} is not {}-byte aligned",
            addr,
            u32::BYTES
        );
        assert_eq!(
            bytes.len() % usize::from(u32::BYTES),
            0,
            "`bytes.len()` is not a multiple of {}",
            u32::BYTES
        );

        if bytes.is_empty() {
            return Ok(());
        }

        info!("MRW {:#010x} <{} bytes>", addr, bytes.len());

        self.banked_data_mode = false;

        self.push_dap_transfer_request(
            adiv5::Register::AHB_AP_CSW,
            dap::Request::Write(CSW_MUST_PRESERVE | CSW_ADDRINC_PACKED | u8::CSW_SIZE),
        );

        let mut total_bytes = bytes.len();
        let mut offset = addr % CSW_ADDRINC_BOUNDARY;
        if offset as usize + total_bytes > CSW_ADDRINC_BOUNDARY as usize {
            let mut addr = addr;

            while total_bytes != 0 {
                // AHB_AP_TAR will need to be updated mid-way to avoid auto-increment wrap-around
                let n = cmp::min(total_bytes, (CSW_ADDRINC_BOUNDARY - offset) as usize);
                offset = 0;
                self.transfer_block_write_csw(addr, &bytes[..n])?;

                bytes = &bytes[n..];
                addr += n as u32;
                total_bytes -= n;
            }
        } else {
            self.transfer_block_write_csw(addr, bytes)?;
        }

        Ok(())
    }

    fn transfer_block_write_csw(
        &mut self,
        mut addr: u32,
        bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
        /// Request Header Size
        const RHS: u16 = 5;

        assert!(
            (addr % CSW_ADDRINC_BOUNDARY) as usize + bytes.len() <= CSW_ADDRINC_BOUNDARY as usize
        );
        assert_eq!(
            bytes.len() % usize::from(u32::BYTES),
            0,
            "`bytes.len()` must be a multiple of {}",
            u32::BYTES
        );

        let words = bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("UNREACHABLE")))
            .collect::<Vec<_>>();
        let mut words = &*words;

        if self.tar.map(|tar| tar != addr).unwrap_or(true) {
            self.push_dap_transfer_request(adiv5::Register::AHB_AP_TAR, dap::Request::Write(addr));
            self.tar = Some(addr);
        }
        self.execute_dap_transfer()?;

        let max_count_per_transfer =
            usize::from(util::round_down(self.packet_size - RHS, u32::BYTES) / u32::BYTES);

        let mut total_count = words.len();
        while total_count != 0 {
            let count = cmp::min(total_count, max_count_per_transfer);
            let n = count * usize::from(u32::BYTES);

            self.transfer_block_write(adiv5::Register::AHB_AP_DRW, &words[..count])?;

            total_count -= count;
            addr += n as u32;
            words = &words[count..];
        }

        if addr % CSW_ADDRINC_BOUNDARY == 0 {
            // wrap-around
            self.tar = Some(addr - CSW_ADDRINC_BOUNDARY);
        } else {
            self.tar = Some(addr);
        }

        Ok(())
    }
}

impl sealed::Data for u8 {
    const BYTES: u16 = 1;
    const CSW_SIZE: u32 = 0b00;

    fn acronym() -> &'static str {
        "B"
    }

    fn push(memory: &mut Vec<Self>, bytes: &[u8], offset: u32, mut count: u16) {
        assert_eq!(bytes.len() % usize::from(u32::BYTES), 0);

        for chunk in bytes.chunks_exact(usize::from(u32::BYTES)) {
            let mut chunk = *array_ref!(chunk, 0, u32::BYTES as usize);
            chunk.rotate_left(offset as usize);

            let len = cmp::min(count, u32::BYTES);
            memory.extend_from_slice(&chunk[..usize::from(len)]);
            count -= len;
        }
    }
}

impl sealed::Data for u16 {
    const BYTES: u16 = 2;
    const CSW_SIZE: u32 = 0b01;

    fn acronym() -> &'static str {
        "H"
    }

    fn push(memory: &mut Vec<Self>, bytes: &[u8], offset: u32, mut count: u16) {
        assert_eq!(bytes.len() % usize::from(u32::BYTES), 0);

        for chunk in bytes.chunks_exact(usize::from(u32::BYTES)) {
            let mut chunk = *array_ref!(chunk, 0, u32::BYTES as usize);
            chunk.rotate_left(offset as usize);

            let len = cmp::min(count * u16::BYTES, u32::BYTES);

            for chunk in chunk[..usize::from(len)].chunks(usize::from(u16::BYTES)) {
                memory.push(Self::from_le_bytes(*array_ref!(
                    chunk,
                    0,
                    u16::BYTES as usize
                )))
            }

            count -= len / u16::BYTES;
        }
    }
}

impl sealed::Data for u32 {
    const BYTES: u16 = 4;
    const CSW_SIZE: u32 = 0b10;

    fn acronym() -> &'static str {
        "W"
    }

    fn push(memory: &mut Vec<Self>, bytes: &[u8], _offset: u32, _count: u16) {
        assert_eq!(bytes.len() % usize::from(u32::BYTES), 0);
        assert_eq!(_offset, 0);

        for i in 0..bytes.len() >> 2 {
            memory.push(Self::from_le_bytes(*array_ref!(bytes, i * 4, 4)))
        }
    }
}
