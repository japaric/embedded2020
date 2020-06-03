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
        endpoint, ia, interface, Direction, Endpoint,
    };

    const PACKET_SIZE: bMaxPacketSize0 = bMaxPacketSize0::B64;
    const CONFIG_VAL: u8 = 1;

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
        let mut bytes = vec![];

        let config = configuration::Descriptor {
            bConfigurationValue: NonZeroU8::new(CONFIG_VAL).unwrap(),
            bMaxPower: 250, // 500 mA
            bNumInterfaces: NonZeroU8::new(2).unwrap(),
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
                bFirstInterface: 0,
                bFunctionClass: comm.class(),
                bFunctionSubClass: comm.subclass(),
                bFunctionProtocol: comm.protocol(),
                bInterfaceCount: NonZeroU8::new(2).unwrap(),
                iFunction: None,
            };

            bytes.extend_from_slice(&ia.bytes());

            let iface0 = interface::Descriptor {
                bAlternativeSetting: 0,
                bInterfaceNumber: 0,
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
                    call_management: false,
                    data_class: false,
                },
                bDataInterface: 1,
            };

            bytes.extend_from_slice(&call.bytes());

            let acm = acm::Descriptor {
                bmCapabilities: acm::Capabilities {
                    comm_features: false,
                    line_serial: false, // ?
                    network_connection: false,
                    send_break: false, // ?
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
                bInterfaceProtocol: 0,
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

        let total_length = bytes.len();
        assert!(
            total_length <= usize::from(u16::max_value()),
            "configuration descriptor is too long"
        );

        bytes[2] = total_length as u8;
        bytes[3] = (total_length >> 8) as u8;

        bytes
    }

    let max_packet_size0 = PACKET_SIZE as u8;
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
            #[link_section = ".data.CONFIG_DESC"]
            static CONFIG_DESC: [u8; #cdl] = [#(#cdb,)*];
            #[link_section = ".data.DEVICE_DESC"]
            static DEVICE_DESC: [u8; #ddl] = [#(#ddb,)*];
        )
        .to_string(),
    )?;

    Ok(())
}
