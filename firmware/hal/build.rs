use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = &PathBuf::from(env::var("OUT_DIR")?);
    let flash = env::var_os("CARGO_FEATURE_FLASH").is_some();

    descs(&out_dir)?;

    // put the linker script somewhere the linker can find it
    fs::copy("interrupts.x", out_dir.join("interrupts.x"))?;
    let suffix = if flash { "flash" } else { "ram" };
    fs::copy(format!("link-{}.x", suffix), out_dir.join("link.x"))?;
    println!("cargo:rustc-link-search={}", out_dir.display());

    Ok(())
}

// generate USB descriptors
fn descs(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    use core::num::NonZeroU8;

    use quote::quote;
    use usb2::{
        cdc::{self, acm, call, header, union},
        configuration::{self, bmAttributes},
        device::{self, bMaxPacketSize0},
        endpoint, hid, ia, interface, Direction, Endpoint,
    };

    const PACKET_SIZE: bMaxPacketSize0 = bMaxPacketSize0::B64;
    const CONFIG_VAL: u8 = 1;
    const CDC_IFACE: u8 = 0;
    const HID_IFACE: u8 = 2;

    let device_desc = device::Descriptor {
        // IAD model
        bDeviceClass: 0xEF,
        bDeviceSubClass: 2,
        bDeviceProtocol: 1,

        bMaxPacketSize0: bMaxPacketSize0::B64,
        bNumConfigurations: NonZeroU8::new(1).unwrap(),
        bcdDevice: 0x01_00,
        iManufacturer: None,
        iProduct: None,
        iSerialNumber: None,
        idProduct: consts::PID,
        idVendor: consts::VID,
    };

    fn full_config_desc() -> Vec<u8> {
        let hid = env::var_os("CARGO_FEATURE_HID").is_some();

        let mut bytes = vec![];

        let mut nifaces = 2;
        if hid {
            nifaces += 1;
        }

        let config = configuration::Descriptor {
            bConfigurationValue: NonZeroU8::new(CONFIG_VAL).unwrap(),
            bMaxPower: 250, // 500 mA
            bNumInterfaces: NonZeroU8::new(nifaces).unwrap(),
            bmAttributes: bmAttributes {
                remote_wakeup: false,
                self_powered: false,
            },
            iConfiguration: None,
            // NOTE this will be fixed at the end of this function
            wTotalLength: 0,
        };

        bytes.extend_from_slice(&config.bytes());

        {
            let comm = cdc::Class::Communications {
                subclass: cdc::SubClass::AbstractControlModel,
                protocol: cdc::Protocol::ATCommands,
            };

            let ia = ia::Descriptor {
                bFirstInterface: CDC_IFACE,
                bFunctionClass: comm.class(),
                bFunctionSubClass: comm.subclass(),
                bFunctionProtocol: comm.protocol(),
                bInterfaceCount: NonZeroU8::new(2).unwrap(),
                iFunction: None,
            };

            bytes.extend_from_slice(&ia.bytes());

            let iface0 = interface::Descriptor {
                bAlternativeSetting: 0,
                bInterfaceNumber: CDC_IFACE,
                bInterfaceClass: comm.class().get(),
                bInterfaceSubClass: comm.subclass(),
                bInterfaceProtocol: comm.protocol(),
                bNumEndpoints: 1,
                iInterface: None,
            };

            bytes.extend_from_slice(&iface0.bytes());

            let header = header::Descriptor { bcdCDC: 0x01_10 };

            bytes.extend_from_slice(&header.bytes());

            let call = call::Descriptor {
                bmCapabilities: call::Capabilities {
                    call_management: true,
                    data_class: true,
                },
                bDataInterface: 1,
            };

            bytes.extend_from_slice(&call.bytes());

            let acm = acm::Descriptor {
                bmCapabilities: acm::Capabilities {
                    comm_features: false,
                    line_serial: true,
                    network_connection: false,
                    send_break: false,
                },
            };

            bytes.extend_from_slice(&acm.bytes());

            let union = union::Descriptor {
                bControlInterface: 0,
                bSubordinateInterface0: 1,
            };

            bytes.extend_from_slice(&union.bytes());

            let ep1in = endpoint::Descriptor {
                bEndpointAddress: Endpoint {
                    direction: Direction::In,
                    number: 1,
                },
                bInterval: 32, // ??
                ty: endpoint::Type::Interrupt {
                    transactions_per_microframe: endpoint::Transactions::_1,
                },
                max_packet_size: PACKET_SIZE as u16,
            };

            bytes.extend_from_slice(&ep1in.bytes());
        }

        {
            let cdc_data = cdc::Class::CdcData;

            let iface1 = interface::Descriptor {
                bAlternativeSetting: 0,
                bInterfaceNumber: 1,
                bInterfaceClass: cdc_data.class().get(),
                bInterfaceSubClass: cdc_data.subclass(),
                bInterfaceProtocol: cdc_data.protocol(),
                bNumEndpoints: 2,
                iInterface: None,
            };

            bytes.extend_from_slice(&iface1.bytes());

            let ep2out = endpoint::Descriptor {
                bEndpointAddress: Endpoint {
                    direction: Direction::Out,
                    number: 2,
                },
                bInterval: 0,
                ty: endpoint::Type::Bulk,
                max_packet_size: PACKET_SIZE as u16,
            };

            bytes.extend_from_slice(&ep2out.bytes());

            let ep2in = endpoint::Descriptor {
                bEndpointAddress: Endpoint {
                    direction: Direction::In,
                    number: 2,
                },
                bInterval: 0,
                ty: endpoint::Type::Bulk,
                max_packet_size: PACKET_SIZE as u16,
            };

            bytes.extend_from_slice(&ep2in.bytes());
        }

        if hid {
            let hid = hid::Class;

            let iface2 = interface::Descriptor {
                bAlternativeSetting: 0,
                bInterfaceNumber: HID_IFACE,
                bInterfaceClass: hid.class().get(),
                bInterfaceSubClass: hid.subclass(),
                bInterfaceProtocol: hid.protocol(),
                bNumEndpoints: 2,
                iInterface: None,
            };

            bytes.extend_from_slice(&iface2.bytes());

            let report = hid::Descriptor {
                bCountryCode: hid::Country::NotSupported,
                wDescriptorLength: 33,
            };

            bytes.extend_from_slice(&report.bytes());

            let ep3out = endpoint::Descriptor {
                bEndpointAddress: Endpoint {
                    direction: Direction::Out,
                    number: 3,
                },
                bInterval: 1,
                ty: endpoint::Type::Interrupt {
                    transactions_per_microframe: endpoint::Transactions::_1,
                },
                max_packet_size: PACKET_SIZE as u16,
            };

            bytes.extend_from_slice(&ep3out.bytes());

            let ep3in = endpoint::Descriptor {
                bEndpointAddress: Endpoint {
                    direction: Direction::In,
                    number: 3,
                },
                bInterval: 1,
                ty: endpoint::Type::Interrupt {
                    transactions_per_microframe: endpoint::Transactions::_1,
                },
                max_packet_size: PACKET_SIZE as u16,
            };

            bytes.extend_from_slice(&ep3in.bytes());
        }

        let total_length = bytes.len();
        assert!(
            total_length <= usize::from(u16::max_value()),
            "configuration descriptor is too long"
        );

        bytes[2] = total_length as u8;
        bytes[3] = (total_length >> 8) as u8;

        bytes
    }

    let line_coding = acm::LineCoding {
        bCharFormat: acm::bCharFormat::Stop1,
        bDataBits: acm::bDataBits::_8,
        bParityType: acm::bParityType::None,
        dwDTERate: 9_600,
    };

    let serial_state = acm::SerialState {
        interface: 0,
        bOverRun: false,
        bParity: false,
        bFraming: false,
        bRingSignal: false,
        bBreak: false,
        bTxCarrier: true,
        bRxCarrier: true,
    };

    let max_packet_size0 = PACKET_SIZE as u8;
    let lcb = line_coding.bytes();
    let lcl = lcb.len();
    let ssb = serial_state.bytes();
    let ssl = ssb.len();
    let ddb = device_desc.bytes();
    let ddl = ddb.len();
    let cdb = full_config_desc();
    let cdl = cdb.len();
    fs::write(
        out_dir.join("descs.rs"),
        quote!(
            const CONFIG_VAL: core::num::NonZeroU8 = unsafe {
                core::num::NonZeroU8::new_unchecked(#CONFIG_VAL)
            };
            const MAX_PACKET_SIZE0: u8 = #max_packet_size0;
            #[allow(dead_code)]
            #[link_section = ".data.CONFIG_DESC"]
            static CONFIG_DESC: [u8; #cdl] = [#(#cdb,)*];

            #[allow(dead_code)]
            #[link_section = ".data.DEVICE_DESC"]
            static DEVICE_DESC: [u8; #ddl] = [#(#ddb,)*];

            #[allow(dead_code)]
            static mut LINE_CODING: [u8; #lcl] = [#(#lcb,)*];

            #[allow(dead_code)]
            #[link_section = ".data.SERIAL_STATE"]
            static SERIAL_STATE: crate::util::Align4<[u8; #ssl]> = crate::util::Align4([#(#ssb,)*]);

            #[allow(dead_code)]
            const CDC_IFACE: u8 = #CDC_IFACE;
            #[allow(dead_code)]
            const HID_IFACE: u8 = #HID_IFACE;
        )
        .to_string(),
    )?;

    Ok(())
}
