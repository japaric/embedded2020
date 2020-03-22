use std::borrow::Cow;

pub struct Device<'a> {
    pub extra_docs: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub peripherals: Vec<Peripheral<'a>>,
}

pub enum Instances<'a> {
    Single { base_address: u64 },
    Many { instances: Vec<Instance<'a>> },
}

pub struct Instance<'a> {
    pub suffix: Cow<'a, str>,
    pub base_address: u64,
}

pub struct Peripheral<'a> {
    pub description: Option<Cow<'a, str>>,
    pub instances: Instances<'a>,
    pub name: Cow<'a, str>,
    pub registers: Vec<Register<'a>>,
}

pub struct Register<'a> {
    pub access: Access,
    pub description: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    pub offset: u64,
    pub r_fields: Vec<Bitfield<'a>>,
    pub w_fields: Vec<Bitfield<'a>>,
    /// In *bytes*; must be one of `[1, 2, 4, 8]`
    pub width: Width,
}

/// Register width
#[derive(Clone, Copy)]
pub enum Width {
    U8,
    U16,
    U32,
    U64,
}

impl Width {
    pub fn bits(self) -> u8 {
        match self {
            Width::U8 => 8,
            Width::U16 => 16,
            Width::U32 => 32,
            Width::U64 => 64,
        }
    }
}

/// Register access
#[derive(Clone, Copy, PartialEq)]
pub enum Access {
    ReadOnly,
    WriteOnly { unsafe_write: bool },
    ReadWrite { unsafe_write: bool },
}

impl Access {
    pub fn can_read(self) -> bool {
        match self {
            Access::ReadOnly | Access::ReadWrite { .. } => true,
            Access::WriteOnly { .. } => false,
        }
    }

    pub fn can_write(self) -> bool {
        match self {
            Access::WriteOnly { .. } | Access::ReadWrite { .. } => true,
            Access::ReadOnly => false,
        }
    }

    pub fn write_is_unsafe(&self) -> bool {
        match self {
            Access::WriteOnly { unsafe_write } | Access::ReadWrite { unsafe_write } => {
                *unsafe_write
            }
            _ => false,
        }
    }

    pub fn make_write_unsafe(&mut self) {
        match self {
            Access::WriteOnly { unsafe_write } | Access::ReadWrite { unsafe_write } => {
                *unsafe_write = true
            }
            _ => panic!("`make_write_unsafe` called on a register with no write access"),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Bitfield<'a> {
    pub description: Option<Cow<'a, str>>,
    pub name: Cow<'a, str>,
    /// In bits; must be less than the register width
    pub offset: u8,
    /// In bits; must be greater than `0` and less than the register width
    pub width: u8,
}

impl Bitfield<'_> {
    pub fn mask(&self) -> u64 {
        (1 << self.width) - 1
    }
}
