#[derive(Clone, Copy)]
pub enum Register {
    // RF control
    RFCON0 = 0x200,
    RFCON1 = 0x201,
    RFCON2 = 0x202,
    RFCON3 = 0x203,
    RFCON5 = 0x205,
    RFCON6 = 0x206,
    RFCON7 = 0x207,
    RFCON8 = 0x208,

    // Sleep calibration
    SLPCAL0 = 0x209,
    SLPCAL1 = 0x20A,
    SLPCAL2 = 0x20B,

    // RF state
    RFSTATE = 0x20F,

    // Averaged RSSI value
    RSSI = 0x210,

    // Sleep clock control
    SLPCON0 = 0x211,
    SLPCON1 = 0x220,

    // Wake-up time match value
    WAKETIMEL = 0x222,
    WAKETIMEH = 0x223,

    // Remain counter
    REMCNTL = 0x224,
    REMCNTH = 0x225,

    // Main counter
    MAINCNT0 = 0x226,
    MAINCNT1 = 0x227,
    MAINCNT2 = 0x228,
    MAINCNT3 = 0x229,

    // Test mode
    TESTMODE = 0x22F,

    // Associated coordinator extended address
    ASSOEADR0 = 0x230,
    ASSOEADR1 = 0x231,
    ASSOEADR2 = 0x232,
    ASSOEADR3 = 0x233,
    ASSOEADR4 = 0x234,
    ASSOEADR5 = 0x235,
    ASSOEADR6 = 0x236,
    ASSOEADR7 = 0x237,

    // Associated coordinator short address
    ASSOSADR0 = 0x238,
    ASSOSADR1 = 0x239,

    // Upper nonce security
    UPNONCE0 = 0x240,
    UPNONCE1 = 0x241,
    UPNONCE2 = 0x242,
    UPNONCE3 = 0x243,
    UPNONCE4 = 0x244,
    UPNONCE5 = 0x245,
    UPNONCE6 = 0x246,
    UPNONCE7 = 0x247,
    UPNONCE8 = 0x248,
    UPNONCE9 = 0x249,
    UPNONCE10 = 0x24A,
    UPNONCE11 = 0x24B,
    UPNONCE12 = 0x24C,
}

impl Register {
    pub(crate) fn opcode(self, act: crate::Action) -> [u8; 2] {
        (((((1 << 10) | self as u16) << 1) | (act as u16)) << 4).to_be_bytes()
    }
}
