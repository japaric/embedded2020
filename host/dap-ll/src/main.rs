// #![deny(warnings)]

use std::{convert::TryInto, fs, path::PathBuf, time::Instant};

use anyhow::anyhow;
use cmsis_dap::{
    cortex_m::{Cpuid, CPUID},
    Dap,
};
use log::info;
use structopt::StructOpt;
use xmas_elf::{sections::SHF_ALLOC, ElfFile};

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long, parse(try_from_str = parse_hex))]
    vendor: u16,

    #[structopt(short, long, parse(try_from_str = parse_hex))]
    product: u16,

    #[structopt(long)]
    verify: bool,

    #[structopt(name = "ELF", parse(from_os_str))]
    elf: PathBuf,
}

fn parse_hex(s: &str) -> Result<u16, anyhow::Error> {
    u16::from_str_radix(s, 16).map_err(|e| e.into())
}

struct Section<'a> {
    address: u32,
    bytes: &'a [u8],
    name: &'a str,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let opts = Opts::from_args();

    let bytes = fs::read(opts.elf)?;
    info!("parsing ELF file");
    let elf = &ElfFile::new(&bytes).map_err(anyhow::Error::msg)?;

    info!("extracting allocatable sections from the ELF file");
    let mut sections = vec![];
    for sect in elf.section_iter() {
        let is_allocatable = sect.flags() & SHF_ALLOC != 0;

        if is_allocatable {
            let name = sect.get_name(elf).map_err(anyhow::Error::msg)?;
            let address = sect.address();
            let size = sect.size();
            let max = u64::from(u32::max_value());
            if address > max || address + size > max {
                return Err(anyhow!(
                    " section `{}` is outside the 32-bit address space",
                    name
                ));
            }

            if address % 4 != 0 || size % 4 != 0 {
                return Err(anyhow!(
                    " section `{}` is not 4-byte aligned (start = {:#010x}, size = {})",
                    name,
                    address,
                    size
                ));
            }

            sections.push(Section {
                address: address as u32,
                bytes: sect.raw_data(elf),
                name,
            })
        }
    }

    let mut dap = Dap::open(opts.vendor, opts.product)?;

    dap.default_swd_configuration()?;

    let cpuid = Cpuid::from(dap.memory_read_word(CPUID)?);
    info!(
        "target: {} (CPUID = {:#010x})",
        cpuid.partno(),
        cpuid.bits()
    );

    dap.cortex_m_halt()?;

    info!("loading ELF into the target's memory");
    for section in sections {
        let start = Instant::now();
        dap.memory_write(section.address, section.bytes)?;
        let end = Instant::now();

        let dur = end - start;
        eprintln!(
            "loaded section `{}` ({} B) in {:?}",
            section.name,
            section.bytes.len(),
            dur
        );

        if opts.verify {
            // verify write
            let start = Instant::now();
            let bytes = dap.memory_read::<u8>(
                section.address,
                section.bytes.len().try_into().expect("UNIMPLEMENTED"),
            )?;

            if bytes != section.bytes {
                return Err(anyhow!("verification of section `{}` failed", section.name));
            }
            let end = Instant::now();

            eprintln!("verified section `{}` in {:?}", section.name, end - start);
        }
    }

    eprintln!("booting program");
    info!("resetting the target");
    dap.cortex_m_sysresetreq()?;

    Ok(())
}
