#![allow(warnings)]
#![deny(unused_result)]
#![no_std]

use core::{future::Future, mem::MaybeUninit, time::Duration};

use hal::{
    spi::{ChipSelect, Spi},
    timer::Timer,
};

mod long;
mod reg;
mod short;

pub struct Mrf24j40 {
    spi: Spi,
    cs: ChipSelect,
}

#[derive(Clone, Copy)]
pub enum Channel {
    /// `2_405` MHz
    _11 = 0,
    /// `2_410` MHz
    _12 = 1,
    /// `2_415` MHz
    _13 = 2,
    /// `2_420` MHz
    _14 = 3,
    /// `2_425` MHz
    _15 = 4,
    /// `2_430` MHz
    _16 = 5,
    /// `2_435` MHz
    _17 = 6,
    /// `2_440` MHz
    _18 = 7,
    /// `2_445` MHz
    _19 = 8,
    /// `2_450` MHz
    _20 = 9,
    /// `2_455` MHz
    _21 = 10,
    /// `2_460` MHz
    _22 = 11,
    /// `2_465` MHz
    _23 = 12,
    /// `2_470` MHz
    _24 = 13,
    /// `2_475` MHz
    _25 = 14,
    /// `2_480` MHz
    _26 = 15,
}

impl Mrf24j40 {
    pub async fn new(spi: Spi, cs: ChipSelect, timer: &mut Timer, channel: Channel) -> Self {
        let mut this = Self { spi, cs };

        /* Initialization as per "Example 3-1" in the data sheet */

        // FIFOEN = 1, TXONTS = 0x6
        this.write_register(reg::PACON2, 0x98).await;

        // RFSTBL = 0x9
        this.write_register(reg::TXSTBL, 0x95).await;

        // RFOPT = 0x03
        this.write_register(reg::RFCON0, 0x03).await;

        // VCOOPT = 0x02 (NOTE "Example 3-1" says RFCON1 = 0x1, but data sheet says that 0x2 is the
        // optimal value)
        this.write_register(reg::RFCON1, 0x02).await;

        // Enable PLL (PLLEN = 1)
        this.write_register(reg::RFCON2, 0x80).await;

        // TXFIL = 1, 20MRECVR = 1
        this.write_register(reg::RFCON6, 0x90).await;

        // SLPCLKSEL = 0x2 (100 KHz internal oscillator)
        this.write_register(reg::RFCON7, 0x80).await;

        // RFVCO = 1
        this.write_register(reg::RFCON8, 0x10).await;

        // CLKOUTEN = 1, SLPCLKDIV = 0x01
        this.write_register(reg::SLPCON1, 0x21).await;

        // the default is 'only carrier sense'
        // CCAMODE = ED (0b10)
        // this.write_register(reg::BBREG2, 0x80).await;

        // set CCA ED threshold to the recommended value
        // this.write_register(reg::CCAEDTH, 0x60).await;

        // append RSSI value to RXFIFO
        this.write_register(reg::BBREG6, 0x40).await;

        this.write_register(reg::RFCON0, ((channel as u8) << 4) | 0x03)
            .await;

        // Reset RF state machine
        this.write_register(reg::RFCTL, 0x04).await;
        this.write_register(reg::RFCTL, 0x00).await;
        timer.wait(Duration::from_micros(192));

        this
    }

    async fn long_read_register(&mut self, reg: long::Register) -> u8 {
        let mut val: [u8; 1] = unsafe { MaybeUninit::uninit().assume_init() };
        self.cs.select();
        self.spi.write(&reg.opcode(Action::Read)).await;
        self.spi.read(&mut val).await;
        self.cs.deselect();
        val[0]
    }

    async fn long_write_register(&mut self, reg: long::Register, val: u8) {
        let opcode = reg.opcode(Action::Write);
        self.cs.select();
        self.spi.write(&[opcode[0], opcode[1], val]).await;
        self.cs.deselect();
    }

    async fn read_register(&mut self, reg: impl Into<Register>) -> u8 {
        match reg.into() {
            Register::Short(reg) => self.short_read_register(reg).await,
            Register::Long(reg) => self.long_read_register(reg).await,
        }
    }

    async fn write_register(&mut self, reg: impl Into<Register>, val: u8) {
        match reg.into() {
            Register::Short(reg) => self.short_write_register(reg, val).await,
            Register::Long(reg) => self.long_write_register(reg, val).await,
        }
    }

    async fn short_read_register(&mut self, reg: short::Register) -> u8 {
        let mut val: [u8; 1] = unsafe { MaybeUninit::uninit().assume_init() };
        self.cs.select();
        self.spi.write(&[reg.opcode(Action::Read)]).await;
        self.spi.read(&mut val).await;
        self.cs.deselect();
        val[0]
    }

    async fn short_write_register(&mut self, reg: short::Register, val: u8) {
        self.cs.select();
        self.spi.write(&[reg.opcode(Action::Write), val]).await;
        self.cs.deselect();
    }
}

pub enum Register {
    Short(short::Register),
    Long(long::Register),
}

impl From<short::Register> for Register {
    fn from(sr: short::Register) -> Self {
        Register::Short(sr)
    }
}

impl From<long::Register> for Register {
    fn from(sr: long::Register) -> Self {
        Register::Long(sr)
    }
}

pub enum Error {}

enum Action {
    Read = 0,
    Write = 1,
}
