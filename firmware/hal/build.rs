use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = &PathBuf::from(env::var("OUT_DIR")?);

    descs(&out_dir)?;

    // put the linker script somewhere the linker can find it
    fs::copy("link.x", out_dir.join("link.x"))?;
    println!("cargo:rustc-link-search={}", out_dir.display());

    Ok(())
}

// generate USB descriptors
fn descs(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    use quote::quote;
    use usb2::{
        cdc::{
            acm, call,
            header::{self, bcdCDC},
            union,
        },
        config,
        device::{self, bMaxPacketSize0, bcdUSB},
        ep::{self, Transactions},
        iface, Direction,
    };

    const PACKET_SIZE: bMaxPacketSize0 = bMaxPacketSize0::B64;

    const DEVICE_DESC: device::Desc = device::Desc {
        bDeviceClass: 0,
        bDeviceSubClass: 0,
        bDeviceProtocol: 0,

        bMaxPacketSize0: bMaxPacketSize0::B64,
        bNumConfigurations: 1,
        bcdDevice: 0x01_00,
        bcdUSB: bcdUSB::V20,
        iManufacturer: 0,
        iProduct: 0,
        iSerialNumber: 0,
        idProduct: consts::PID,
        idVendor: consts::VID,
    };

    fn full_config_desc() -> Vec<u8> {
        let mut bytes = vec![];

        const CONFIG_DESC: config::Desc = config::Desc {
            bConfigurationValue: 1,
            bMaxPower: 250, // 500 mA
            bNumInterfaces: 2,
            bmAttributes: config::bmAttributes {
                remote_wakeup: false,
                self_powered: true,
            },
            iConfiguration: 0,
            // NOTE this will be fixed at the end of this function
            wTotalLength: 0,
        };

        bytes.extend_from_slice(&CONFIG_DESC.bytes());

        const IFACE0_DESC: iface::Desc = iface::Desc {
            bAlternativeSetting: 0,
            bInterfaceNumber: 0,
            bInterfaceClass: iface::Class::Communications {
                subclass: iface::CommunicationsSubclass::Acm,
            },
            bInterfaceProtocol: 0,
            bNumEndpoints: 1,
            iInterface: 0,
        };

        bytes.extend_from_slice(&IFACE0_DESC.bytes());

        // functional descriptors
        const HEADER_DESC: header::Desc = header::Desc {
            bcdCDC: bcdCDC::V11,
        };

        bytes.extend_from_slice(&HEADER_DESC.bytes());

        const ACM_DESC: acm::Desc = acm::Desc {
            bmCapabilities: acm::Capabilities {
                comm_features: false,
                line_serial: false,
                network_connection: false,
                send_break: false,
            },
        };

        bytes.extend_from_slice(&ACM_DESC.bytes());

        const UNION_DESC: union::Desc = union::Desc {
            bControlInterface: 0,
            bSubordinateInterface0: 1,
        };

        bytes.extend_from_slice(&UNION_DESC.bytes());

        const CALL_DESC: call::Desc = call::Desc {
            bmCapabilities: call::Capabilities {
                call_management: true,
                data_class: true,
            },
            bDataInterface: 1,
        };

        bytes.extend_from_slice(&CALL_DESC.bytes());

        const EPIN2_DESC: ep::Desc = ep::Desc {
            bEndpointAddress: ep::Address {
                direction: Direction::IN,
                number: 2,
            },
            bInterval: 16, // ??
            bmAttributes: ep::bmAttributes::Interrupt,
            wMaxPacketSize: ep::wMaxPacketSize::IsochronousInterrupt {
                size: PACKET_SIZE as u16,
                transactions_per_microframe: Transactions::_1,
            },
        };

        bytes.extend_from_slice(&EPIN2_DESC.bytes());

        const IFACE1_DESC: iface::Desc = iface::Desc {
            bAlternativeSetting: 0,
            bInterfaceNumber: 1,
            bInterfaceClass: iface::Class::CdcData,
            bInterfaceProtocol: 0,
            bNumEndpoints: 2,
            iInterface: 0,
        };

        bytes.extend_from_slice(&IFACE1_DESC.bytes());

        const EPIN1_DESC: ep::Desc = ep::Desc {
            bEndpointAddress: ep::Address {
                direction: Direction::IN,
                number: 1,
            },
            bInterval: 0,
            bmAttributes: ep::bmAttributes::Bulk,
            wMaxPacketSize: ep::wMaxPacketSize::BulkControl {
                size: PACKET_SIZE as u16,
            },
        };

        bytes.extend_from_slice(&EPIN1_DESC.bytes());

        const EPOUT1_DESC: ep::Desc = ep::Desc {
            bEndpointAddress: ep::Address {
                direction: Direction::OUT,
                number: 1,
            },
            bInterval: 0,
            bmAttributes: ep::bmAttributes::Bulk,
            wMaxPacketSize: ep::wMaxPacketSize::BulkControl {
                size: PACKET_SIZE as u16,
            },
        };

        bytes.extend_from_slice(&EPOUT1_DESC.bytes());

        let total_length = bytes.len();
        assert!(
            total_length <= u16::max_value() as usize,
            "configuration descriptor is too long"
        );

        bytes[2] = total_length as u8;
        bytes[3] = (total_length >> 8) as u8;

        bytes
    }

    let max_packet_size0 = PACKET_SIZE as u8;
    let ddb = DEVICE_DESC.bytes();
    let ddl = ddb.len();
    let cdb = full_config_desc();
    let cdl = cdb.len();
    fs::write(
        out_dir.join("descs.rs"),
        quote!(
            const MAX_PACKET_SIZE0: u8 = #max_packet_size0;
            static DEVICE_DESC: [u8; #ddl] = [#(#ddb,)*];
            static CONFIG_DESC: [u8; #cdl] = [#(#cdb,)*];
        )
        .to_string(),
    )?;

    Ok(())
}
