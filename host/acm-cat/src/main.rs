use core::time::Duration;
use std::{
    io::{self, Read as _, Write as _},
    thread,
};

use anyhow::anyhow;
use serialport::SerialPortType;

fn main() -> Result<(), anyhow::Error> {
    let dev = serialport::available_ports()?
        .into_iter()
        .filter(|info| match info.port_type {
            SerialPortType::UsbPort(ref port) => port.vid == consts::VID,
            _ => false,
        })
        .next()
        .ok_or_else(|| anyhow!("device not found"))?;

    let mut port = serialport::open(&dev.port_name)?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut buf = [0; 64];
    loop {
        if port.bytes_to_read()? != 0 {
            let n = port.read(&mut buf)?;
            stdout.write(&buf[..n])?;
        } else {
            thread::sleep(Duration::from_millis(1))
        }
    }
}
