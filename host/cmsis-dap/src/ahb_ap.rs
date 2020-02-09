use core::cmp;
use std::convert::TryInto as _;

use arrayref::array_ref;
use log::info;

use crate::{
    adiv5, dap,
    sealed::{self, Data as _},
    util,
};

// XXX do other Cortex-M variants have a different "must preserve" value
// see section 8.2.2 of Cortex-M4 TRM (Debug ARM DDI 0439B)
const CSW_MUST_PRESERVE: u32 = (1 << 29) // MasterType
    | (1 << 25) // HPROT1
    | (1 << 24) // Reserved
    | (1 << 6) // DbgStatus
    ;
const CSW_ADDRINC_PACKED: u32 = 0b10 << 4;
const CSW_ADDRINC_BOUNDARY: u32 = 0x400;

impl crate::Dap {
    /// Reads a single word from the target's memory using the AHB-AP (Access Port)
    pub fn memory_read_word(
        &mut self,
        addr: u32,
    ) -> Result<u32, anyhow::Error> {
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
        self.push_dap_transfer_request(
            adiv5::Register::banked_data(addr),
            dap::Request::Read,
        );

        let word = self.execute_dap_transfer()?[0];

        info!("... MRW {:#010x} -> {:#010x}", addr, word);

        Ok(word)
    }

    /// Writes a single word into the target's memory using the AHB-AP (Access Port)
    pub fn memory_write_word(
        &mut self,
        addr: u32,
        val: u32,
    ) -> Result<(), anyhow::Error> {
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
    pub fn memory_read<T>(
        &mut self,
        addr: u32,
        count: u32,
    ) -> Result<Vec<T>, anyhow::Error>
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
            dap::Request::Write(
                CSW_MUST_PRESERVE | CSW_ADDRINC_PACKED | T::CSW_SIZE,
            ),
        );

        let mut memory = vec![];

        let mut total_bytes = count * u32::from(T::BYTES);
        let mut offset = addr % CSW_ADDRINC_BOUNDARY;
        if offset + total_bytes > CSW_ADDRINC_BOUNDARY {
            // AHB_AP_TAR will need to be updated mid-way to avoid auto-increment wrap-around
            let mut addr = addr;

            while total_bytes != 0 {
                let bytes =
                    cmp::min(total_bytes, CSW_ADDRINC_BOUNDARY - offset);
                offset = 0;
                self.transfer_block_read_csw(
                    addr,
                    bytes / u32::from(T::BYTES),
                    &mut memory,
                )?;

                addr += bytes;
                total_bytes -= bytes;
            }
        } else {
            self.transfer_block_read_csw(addr, count, &mut memory)?;
        }

        Ok(memory)
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
            addr % CSW_ADDRINC_BOUNDARY + total_count * u32::from(T::BYTES)
                <= CSW_ADDRINC_BOUNDARY
        );

        if self.tar.map(|tar| tar != addr).unwrap_or(true) {
            self.push_dap_transfer_request(
                adiv5::Register::AHB_AP_TAR,
                dap::Request::Write(addr),
            );
            self.tar = Some(addr);
        }
        self.execute_dap_transfer()?;

        let max_count_per_transfer =
            util::round_down(self.packet_size - RHS, u32::BYTES) / T::BYTES;

        while total_count != 0 {
            // NOTE(as u16) `max_count_per_transfer` has type `u16`
            let count =
                cmp::min(total_count, u32::from(max_count_per_transfer)) as u16;
            let payload_bytes = util::round_up(count * T::BYTES, u32::BYTES);

            let data = self.transfer_block_read(
                adiv5::Register::AHB_AP_DRW,
                payload_bytes / u32::BYTES,
            )?;

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
    pub fn memory_write(
        &mut self,
        addr: u32,
        mut bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
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
            dap::Request::Write(
                CSW_MUST_PRESERVE | CSW_ADDRINC_PACKED | u8::CSW_SIZE,
            ),
        );

        let mut total_bytes = bytes.len();
        let mut offset = addr % CSW_ADDRINC_BOUNDARY;
        if offset as usize + total_bytes > CSW_ADDRINC_BOUNDARY as usize {
            let mut addr = addr;

            while total_bytes != 0 {
                // AHB_AP_TAR will need to be updated mid-way to avoid auto-increment wrap-around
                let n = cmp::min(
                    total_bytes,
                    (CSW_ADDRINC_BOUNDARY - offset) as usize,
                );
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
            (addr % CSW_ADDRINC_BOUNDARY) as usize + bytes.len()
                <= CSW_ADDRINC_BOUNDARY as usize
        );
        assert_eq!(
            bytes.len() % usize::from(u32::BYTES),
            0,
            "`bytes.len()` must be a multiple of {}",
            u32::BYTES
        );

        let words = bytes
            .chunks_exact(4)
            .map(|chunk| {
                u32::from_le_bytes(chunk.try_into().expect("UNREACHABLE"))
            })
            .collect::<Vec<_>>();
        let mut words = &*words;

        if self.tar.map(|tar| tar != addr).unwrap_or(true) {
            self.push_dap_transfer_request(
                adiv5::Register::AHB_AP_TAR,
                dap::Request::Write(addr),
            );
            self.tar = Some(addr);
        }
        self.execute_dap_transfer()?;

        let max_count_per_transfer = usize::from(
            util::round_down(self.packet_size - RHS, u32::BYTES) / u32::BYTES,
        );

        let mut total_count = words.len();
        while total_count != 0 {
            let count = cmp::min(total_count, max_count_per_transfer);
            let n = count * usize::from(u32::BYTES);

            self.transfer_block_write(
                adiv5::Register::AHB_AP_DRW,
                &words[..count],
            )?;

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

            for chunk in
                chunk[..usize::from(len)].chunks(usize::from(u16::BYTES))
            {
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
