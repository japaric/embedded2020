use core::{str, time::Duration};
use std::env;

use anyhow::{ensure, format_err};

const TIMEOUT: Duration = Duration::from_millis(10);
const EPOUT1: u8 = 0x01;
const EPIN1: u8 = 0x81;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>(); // skip program name
    ensure!(args.len() > 0, "expected at least one argument");

    let mut dev = rusb::open_device_with_vid_pid(consts::VID, consts::PID).ok_or_else(|| {
        format_err!(
            "device {:04x}:{:04x} not found or cannot be opened",
            consts::VID,
            consts::PID
        )
    })?;
    dev.claim_interface(0)?;

    ensure!(
        args.iter().all(|arg| arg.len() <= 64),
        "one message is too long (max=64B)"
    );
    for arg in &args {
        dev.write_bulk(EPOUT1, arg.as_bytes(), TIMEOUT)?; // send packets back to back
    }

    let mut buf = [0; 64];
    for _ in 0..args.len() {
        let n = dev.read_bulk(EPIN1, &mut buf, TIMEOUT)?;
        let ans = &buf[..n];
        if let Ok(s) = str::from_utf8(ans) {
            println!("{}", s);
        } else {
            println!("{:?}", ans);
        }
    }

    Ok(())
}
