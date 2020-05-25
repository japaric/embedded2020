use crate::Action;

#[derive(Clone, Copy)]
pub enum Register {
    // Receive MAC control
    RXMCR = 0x00,

    // PAN ID
    PANIDL = 0x01,
    PANIDH = 0x02,

    // Short Address
    SADRL = 0x03,
    SADRH = 0x04,

    // 64-bit extended address
    EADR0 = 0x05,
    EADR1 = 0x06,
    EADR2 = 0x07,
    EADR3 = 0x08,
    EADR4 = 0x09,
    EADR5 = 0x0A,
    EADR6 = 0x0B,
    EADR7 = 0x0C,

    // Receive FIFO flush
    RXFLUSH = 0x0D,

    // Beacon and superframe order
    ORDER = 0x10,

    // CSMA-CA mode control
    TXMCR = 0x11,

    // MAC ACK time-out duration
    ACKTMOUT = 0x12,

    // GTS1 and cap end slot
    ESLOTG1 = 0x13,

    // Symbol period tick
    SYMTICKL = 0x14,
    SYMTICKH = 0x15,

    // Power amplifier control
    PACON0 = 0x16,
    PACON1 = 0x17,
    PACON2 = 0x18,

    // Transmit beacon FIFO control 0
    TXBCON0 = 0x1A,

    // Transmit normal FIFO control
    TXNCON = 0x1B,

    // GTS1 FIFO control
    TXG1CON = 0x1C,

    // GTS2 FIFO control
    TXG2CON = 0x1D,

    // End slot of GTS3 and GTS2
    ESLOTG23 = 0x1E,

    // End slot of GTS5 and GTS4
    ESLOTG45 = 0x1F,

    // End slot of GTS6
    ESLOTG67 = 0x20,

    // TX data pending
    TXPEND = 0x21,

    // Wake control
    WAKECON = 0x22,

    // Superframe counter offset to align beacon
    FRMOFFSET = 0x23,

    // TX MAC status
    TXSTAT = 0x24,

    // Transmit beacon FIFO control 1
    TXBCON1 = 0x25,

    // Gated clock control
    GATECLK = 0x26,

    // TX turnaround time
    TXTIME = 0x27,

    // Half symbol timer
    HSYMTMRL = 0x28,
    HSYMTMRH = 0x29,

    // Software reset
    SOFTRST = 0x2A,

    // Security control
    SECCON0 = 0x2C,
    SECCON1 = 0x2D,

    // TX stabilization
    TXSTBL = 0x2E,

    // RX MAC status
    RXSR = 0x30,

    // Interrupt status
    INTSTAT = 0x31,

    // Interrupt control
    INTCON = 0x32,

    // GPIO port
    GPIO = 0x33,

    // GPIO pin direction
    TRISGPIO = 0x34,

    // Sleep acknowledgment and wake-up counter
    SLPACK = 0x35,

    // RF mode control
    RFCTL = 0x36,

    // Security control 2
    SECCR2 = 0x37,

    // Baseband
    BBREG0 = 0x38,
    BBREG1 = 0x39,
    BBREG2 = 0x3A,
    BBREG3 = 0x3B,
    BBREG4 = 0x3C,
    BBREG6 = 0x3E,

    // Energy detection for CCA
    CCAEDTH = 0x3F,
}

impl Register {
    pub(crate) fn opcode(self, action: Action) -> u8 {
        ((self as u8) << 1) | action as u8
    }
}
