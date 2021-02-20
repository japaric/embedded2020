#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]

// TODO split most modules into library

pub mod cm;
pub mod codegen;
mod fmt;
pub mod ir;
pub mod opt;
mod translate;
mod verify;

use std::{fs, path::Path};

fn main() -> Result<(), anyhow::Error> {
    gen_cm(Path::new("../../../shared/cm/src/lib.rs"))?;
    gen_nrf52(Path::new("../../../firmware/pac/src/lib.rs"))?;

    Ok(())
}

// Audited register writes
const AUDITED: &[&str] = &[
    "CLOCK", "FICR", "P0", "POWER", "RADIO", "RTC0", "TWIM0", "USBD", "SPIM0",
];

fn gen_nrf52(lib: &Path) -> Result<(), anyhow::Error> {
    let xml = fs::read_to_string("nrf52840.svd")?;
    let dev = svd_parser::parse(&xml)?;
    let mut dev = translate::svd::device(&dev, AUDITED);
    audit_nrf52(&mut dev);
    gen(dev, lib)
}

fn audit_nrf52(dev: &mut ir::Device<'_>) {
    for periph in &mut dev.peripherals {
        // all peripherals
        for reg in &mut periph.registers {
            match &*reg.name {
                // enabling interrupts can break critical sections
                "INTEN" | "INTENSET" => {
                    reg.access.make_write_unsafe();
                }
                _ => {}
            }
        }

        // Fix bitfield widths to match the OPS
        if periph.name == "TWIM0" {
            for reg in &mut periph.registers {
                match &*reg.name {
                    "RXD_AMOUNT" | "TXD_AMOUNT" => {
                        for field in reg.r_fields.iter_mut().chain(&mut reg.w_fields) {
                            if field.name == "AMOUNT" {
                                field.width = 8;
                            }
                        }
                    }

                    "RXD_MAXCNT" | "TXD_MAXCNT" => {
                        // DMA related
                        reg.access.make_write_unsafe();

                        for field in reg.r_fields.iter_mut().chain(&mut reg.w_fields) {
                            if field.name == "MAXCNT" {
                                field.width = 8;
                            }
                        }
                    }

                    // DMA related
                    "TASKS_STARTRX" | "TASKS_STARTTX" | "RXD_PTR" | "TXD_PTR" => {
                        reg.access.make_write_unsafe();
                    }

                    _ => {}
                }
            }
        }
    }
}

fn gen_cm(lib: &Path) -> Result<(), anyhow::Error> {
    let dev = cm::device();
    gen(dev, lib)
}

fn gen(mut dev: ir::Device<'_>, lib: &Path) -> Result<(), anyhow::Error> {
    assert!(lib.is_file());

    dev.verify()?;
    opt::device(&mut dev);
    let krate = codegen::device(&dev);
    fs::write(lib, krate)?;
    Ok(())
}
