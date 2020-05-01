//! Abstract Control Management functional descriptor

/// Abstract Control Management functional descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // bFunctionLength: u8,
    // bDescriptorType: u8,
    // bDescriptorSubtype: u8,
    /// Capabilities
    pub bmCapabilities: Capabilities,
}

/// Capabilities
#[derive(Clone, Copy)]
pub struct Capabilities {
    /// Device supports `{Set,Clear,Get}_Comm_Feature`
    pub comm_features: bool,
    /// Device supports `{Set,Get}_Line_Coding`, `Set_Control_Line_State` and `Serial_State`
    pub line_serial: bool,
    /// Device supports `Send_Break`
    pub send_break: bool,
    /// Device supports `Network_Connection`
    pub network_connection: bool,
}

impl Capabilities {
    fn byte(&self) -> u8 {
        let mut byte = 0;
        if self.comm_features {
            byte |= 1 << 0;
        }
        if self.line_serial {
            byte |= 1 << 1;
        }
        if self.send_break {
            byte |= 1 << 2;
        }
        if self.network_connection {
            byte |= 1 << 3;
        }
        byte
    }
}

impl Desc {
    /// Size of this descriptor on the wire
    pub const SIZE: u8 = 4;

    /// Returns the wire representation of this device endpoint
    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            super::CS_INTERFACE,
            super::SUBTYPE_ACM,
            self.bmCapabilities.byte(),
        ]
    }
}
