#![no_std]

/// Precedes a `.symtab` compressed string
pub const UTF8_SYMTAB_STRING: u8 = 0xC0;

/// Precedes a 32.768 KHz timestamp
pub const UTF8_TIMESTAMP: u8 = 0xC1;
