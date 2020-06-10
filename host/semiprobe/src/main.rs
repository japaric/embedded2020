#![allow(warnings)]

use core::{
    cmp,
    convert::TryFrom,
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};
use std::{
    collections::{btree_map, BTreeMap},
    fs,
    io::{self, Write as _},
    mem,
    path::PathBuf,
    process,
    time::Instant,
};

use anyhow::{anyhow, bail};
use arrayref::array_ref;
use cm::scb::{cpuid, CPUID};
use cmsis_dap::cortex_m;
use gimli::{
    read::{CfaRule, DebugFrame, UnwindSection},
    BaseAddresses, EndianSlice, LittleEndian, RegisterRule, UninitializedUnwindContext,
};
use log::{debug, error, info};
use probe_rs::{Core, MemoryInterface as _, Probe};
use structopt::StructOpt;
use xmas_elf::{
    sections::{SectionData, ShType, SHF_ALLOC},
    symbol_table::Entry,
    ElfFile,
};

#[derive(StructOpt)]
struct Opts {
    #[structopt(name = "ELF", parse(from_os_str))]
    elf: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    process::exit(not_main()?)
}

fn not_main() -> Result<i32, anyhow::Error> {
    let beginning = Instant::now();
    env_logger::init();

    let opts = Opts::from_args();

    let bytes = fs::read(opts.elf)?;
    debug!("parsing ELF file");
    let elf = &ElfFile::new(&bytes).map_err(anyhow::Error::msg)?;

    debug!("extracting allocatable sections from the ELF file");
    let mut vectors = None;
    let mut footprints = BTreeMap::new();
    let mut sections = vec![];
    let mut ncursors = 0;
    let mut semidap_cursor = None;
    let mut semidap_buffer = None;
    let mut debug_frame = None;
    let mut range_names = vec![];
    let binfmt_shndx = elf
        .section_iter()
        .zip(0..)
        .filter_map(|(sect, shndx)| {
            if sect.get_name(elf) == Ok(".binfmt") {
                Some(shndx)
            } else {
                None
            }
        })
        .next();
    let text_shndx = elf
        .section_iter()
        .zip(0..)
        .filter_map(|(sect, shndx)| {
            if sect.get_name(elf) == Ok(".text") {
                Some(shndx)
            } else {
                None
            }
        })
        .next();
    for sect in elf.section_iter() {
        let is_allocatable = sect.flags() & SHF_ALLOC != 0;

        let size = sect.size();
        if is_allocatable && size != 0 {
            // NOLOAD section like `.uninit` or `.bss`
            if sect.get_type() == Ok(ShType::NoBits) {
                // we never load these sections
                continue;
            }

            let name = sect.get_name(elf).map_err(anyhow::Error::msg)?;

            let address = sect.address();
            let max = u64::from(u32::max_value());
            if address > max || address + size > max {
                return Err(anyhow!(
                    " section `{}` is outside the 32-bit address space",
                    name
                ));
            }

            let align = mem::size_of::<u32>() as u64;
            if address % align != 0 {
                return Err(anyhow!(
                    " section `{}` is not 4-byte aligned (start = {:#010x}, size = {})",
                    name,
                    address,
                    size
                ));
            }

            let bytes = sect.raw_data(elf);
            if name == ".vectors" {
                let sp = u32::from_le_bytes(*array_ref!(bytes, 0, 4));
                let pc = u32::from_le_bytes(*array_ref!(bytes, 4, 4));

                vectors = Some(Vectors {
                    vtor: address as u32,
                    pc,
                    sp,
                })
            }

            sections.push(Section {
                address: address as u32,
                bytes,
                name,
            })
        } else if sect.get_name(elf) == Ok(".symtab") {
            if let Ok(symtab) = sect.get_data(elf) {
                if let SectionData::SymbolTable32(entries) = symtab {
                    for entry in entries {
                        if let Ok(name) = entry.get_name(elf) {
                            if Some(entry.shndx() as u32) == binfmt_shndx {
                                footprints.insert(entry.value(), name);
                            } else if Some(entry.shndx() as u32) == text_shndx && entry.size() != 0
                            {
                                // clear the thumb bit
                                let mut name = rustc_demangle::demangle(name).to_string();
                                let start = entry.value() & !1;

                                // strip the hash (e.g. `::hd881d91ced85c2b0`)
                                let hash_len = "::hd881d91ced85c2b0".len();
                                if let Some(pos) = name.len().checked_sub(hash_len) {
                                    let maybe_hash = &name[pos..];
                                    if maybe_hash.starts_with("::h") {
                                        // FIXME do not allocate again
                                        name = name[..pos].to_string();
                                    }
                                }

                                range_names.push((start..start + entry.size(), name));
                            }

                            if name == "SEMIDAP_CURSOR" {
                                if let Ok(addr) = u32::try_from(entry.value()) {
                                    ncursors = entry.size() / 2;
                                    semidap_cursor = Some(addr);
                                }
                            } else if name == "SEMIDAP_BUFFER" {
                                let size = entry.size();
                                if size.is_power_of_two() {
                                    if let (Ok(addr), Ok(len)) =
                                        (u32::try_from(entry.value()), u32::try_from(size))
                                    {
                                        semidap_buffer = Some((addr, len));
                                    }
                                } else {
                                    error!("malformed SEMIDAP_BUFFER (len={})", size);
                                }
                            }
                        }
                    }
                }
            }
        } else if sect.get_name(elf) == Ok(".debug_frame") {
            let mut df = DebugFrame::new(sect.raw_data(elf), LittleEndian);
            // 32-bit ARM
            df.set_address_size(4);
            debug_frame = Some(df);
        }
    }

    let vectors = vectors.ok_or_else(|| anyhow!("`.vectors` section not found"))?;

    range_names.sort_unstable_by(|a, b| a.0.start.cmp(&b.0.start));

    let probes = Probe::list_all();
    if probes.is_empty() {
        bail!("no probe is connected")
    }
    let mut probe = probes[0].open()?;
    let mut session = probe.attach("nrf52")?;
    let mut core = session.core(0)?;

    let debug_frame = debug_frame.ok_or_else(|| anyhow!("`.debug_frame` section is missing"))?;

    debug!("resetting and halting the target");
    core.reset_and_halt()?;

    debug!("loading ELF into the target's memory");
    let mut total_bytes = 0;
    let start = Instant::now();
    for section in sections {
        let start = Instant::now();
        core.write_8(section.address, section.bytes)?;
        let end = Instant::now();
        let bytes = section.bytes.len();
        total_bytes += bytes as u64;

        let dur = end - start;
        info!("loaded `{}` ({} B) in {:?}", section.name, bytes, dur);
    }

    let end = Instant::now();

    let dur = end - start;
    const NANOS: u64 = 1_000_000_000;
    let speed = total_bytes * NANOS / (dur.as_secs() * NANOS + u64::from(dur.subsec_nanos()));
    info!("loaded {} bytes in {:?} ({} B/s)", total_bytes, dur, speed);

    core.write_core_reg((cortex_m::Register::LR as u16).into(), LR_END)?;
    core.write_core_reg((cortex_m::Register::SP as u16).into(), vectors.sp)?;
    core.write_core_reg((cortex_m::Register::PC as u16).into(), vectors.pc)?;
    core.write_word_32(cm::scb::VTOR::address() as u32, vectors.vtor)?;

    info!("booting program (start to end: {:?})", end - beginning);

    core.run()?;

    static CONTINUE: AtomicBool = AtomicBool::new(true);
    let mut twice = false;
    let mut observed_empty;
    let mut stdout_buffers = (0..ncursors).map(|_| vec![]).collect::<Vec<_>>();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    // read cursors
    let mut reads: Vec<u16> = (0..ncursors).map(|_| 0).collect();
    // do proper clean-up on Ctrl-C
    ctrlc::set_handler(|| CONTINUE.store(false, Ordering::Relaxed))?;
    let mut last_ts = None;
    while CONTINUE.load(Ordering::Relaxed) {
        fn drain(
            cursorp: u32,
            bufferp: u32,
            total_len: u32,
            readps: &mut [u16],
            hbuffers: &mut [Vec<u8>],
            core: &mut Core,
        ) -> Result</* observed_empty */ bool, anyhow::Error> {
            let mut observed_empty = true;
            let len = (total_len / readps.len() as u32) as u16;
            for i in 0..readps.len() {
                let writep = cursorp + (mem::size_of::<u16>() * i) as u32;
                let bufp = bufferp + (len as usize * i) as u32;
                let readp = &mut readps[i];

                let write = read_word_16(core, writep)?;
                let available = write.wrapping_sub(*readp);
                let bytes = if available == 0 {
                    // no new data
                    continue;
                } else if available >= len {
                    core.reset_and_halt()?;
                    bail!("semidap buffer has been overrun (read={}, write={}) -- reset-halting device", *readp, write);
                } else {
                    let mut bytes = vec![0; available as usize];
                    let cursor = *readp % len;
                    if cursor + available > len {
                        // split memcpy
                        let pivot = len.wrapping_sub(cursor);
                        core.read_8(bufp + u32::from(cursor), &mut bytes[..pivot.into()])?;
                        core.read_8(bufp, &mut bytes[pivot.into()..])?;
                    } else {
                        // single memcpy
                        core.read_8(bufp + u32::from(cursor), &mut bytes)?;
                    }
                    let write = read_word_16(core, writep)?;
                    if write.wrapping_sub(*readp) >= len {
                        core.reset_and_halt()?;
                        bail!("semidap buffer has been overrun (read={}, write={}) -- reset-halting device", *readp, write);
                    }
                    bytes
                };

                observed_empty = false;
                hbuffers[i].extend_from_slice(&bytes);
                *readp = *readp + bytes.len() as u16;
            }

            Ok(observed_empty)
        }

        if let (Some(cursor), Some((bufferp, total_len))) = (semidap_cursor, semidap_buffer) {
            observed_empty = drain(
                cursor,
                bufferp,
                total_len,
                &mut reads,
                &mut stdout_buffers,
                &mut core,
            )?;

            let mut messages = vec![];
            for (src, stdout_buffer) in stdout_buffers.iter_mut().enumerate() {
                if stdout_buffer.is_empty() {
                    continue;
                }

                let mut consumed = 0;
                let mut bytes = &stdout_buffer[..];
                let total = bytes.len();

                debug!("{}> {:?}", src, bytes);
                while let Ok((message, i)) = binfmt_parser::parse_message(&bytes, &footprints) {
                    consumed += i;
                    bytes = &bytes[i..];
                    messages.push((src, message));
                }

                if consumed == total {
                    stdout_buffer.clear();
                } else {
                    *stdout_buffer = stdout_buffer[consumed..].to_owned();
                }
            }

            // FIXME this will still result in unordered messages
            messages.sort_by_key(|(_, m)| m.timestamp);

            for (src, mut message) in messages {
                let curr = message.absolute();
                if let Some(last) = last_ts {
                    message.delta(last);
                }
                writeln!(stdout, "{}>{}", src, message)?;
                last_ts = curr;
            }
        } else {
            observed_empty = true;
        }

        // only handle a syscall when the device is halted, but first try to
        // drain the buffer
        if observed_empty {
            if core.core_halted()? {
                if twice {
                    return handle_syscall(&mut core, &debug_frame, &range_names);
                } else {
                    twice = true;
                }
            }
        }
    }

    core.reset_and_halt()?;

    Ok(0)
}

// the reset value of the Link Register; this indicates the end of the stack
const LR_END: u32 = 0xFFFF_FFFF;

struct Vectors {
    vtor: u32,
    sp: u32,
    pc: u32,
}

struct Section<'a> {
    address: u32,
    bytes: &'a [u8],
    name: &'a str,
}

fn parse_hex(s: &str) -> Result<u16, anyhow::Error> {
    u16::from_str_radix(s, 16).map_err(|e| e.into())
}

fn handle_syscall(
    core: &mut Core,
    debug_frame: &DebugFrame<EndianSlice<LittleEndian>>,
    range_names: &[(Range<u64>, String)],
) -> Result<i32, anyhow::Error> {
    const SYS_ABORT: u16 = 0xbeaa; // BKPT 0xAA
    const SYS_EXCEPTION: u16 = 0xbeff; // BKPT 0xFF
    const SYS_EXIT: u16 = 0xbeab; // BKPT 0xAB

    let pc = core.read_core_reg(cortex_m::Register::PC as u16)?;
    let insn = u16::from(core.read_word_8(pc)?) | u16::from(core.read_word_8(pc + 1)?) << 8;

    match insn {
        SYS_EXIT => {
            let r0 = core.read_core_reg(cortex_m::Register::R0 as u16)?;
            Ok(r0 as i32)
        }

        SYS_EXCEPTION => Ok(1),

        SYS_ABORT => {
            let sp = core.read_core_reg(cortex_m::Register::SP as u16)?;
            let lr = core.read_core_reg(cortex_m::Register::LR as u16)?;
            backtrace(core, debug_frame, range_names, lr, pc, sp)?;
            Ok(134)
        }

        _ => {
            error!("unknown instruction: {:#06x}", insn);
            Ok(1)
        }
    }
}

fn backtrace(
    core: &mut Core,
    debug_frame: &DebugFrame<EndianSlice<LittleEndian>>,
    range_names: &[(Range<u64>, String)],
    lr: u32,
    mut pc: u32,
    sp: u32,
) -> Result<(), anyhow::Error> {
    fn gimli2cortex(reg: &gimli::Register) -> cortex_m::Register {
        if reg.0 == 13 {
            Register::SP
        } else if reg.0 == 14 {
            Register::LR
        } else if reg.0 == 11 {
            Register::R11
        } else if reg.0 == 10 {
            Register::R10
        } else if reg.0 == 9 {
            Register::R9
        } else if reg.0 == 8 {
            Register::R8
        } else if reg.0 == 7 {
            Register::R7
        } else if reg.0 == 6 {
            Register::R6
        } else if reg.0 == 5 {
            Register::R5
        } else if reg.0 == 4 {
            Register::R4
        } else {
            panic!("unknown: {:?}", reg);
        }
    }

    // Lazily evaluated registers
    #[derive(Debug, Default)]
    struct Registers {
        cache: BTreeMap<Register, u32>,
    }

    impl Registers {
        fn new(lr: u32, sp: u32) -> Self {
            let mut cache = BTreeMap::new();
            cache.insert(Register::LR, lr);
            cache.insert(Register::SP, sp);
            Self { cache }
        }

        fn get(&mut self, reg: cortex_m::Register, core: &mut Core) -> Result<u32, anyhow::Error> {
            Ok(match self.cache.entry(reg) {
                btree_map::Entry::Occupied(entry) => *entry.get(),
                btree_map::Entry::Vacant(entry) => *entry.insert(core.read_core_reg(reg as u16)?),
            })
        }

        fn insert(&mut self, reg: cortex_m::Register, val: u32) {
            self.cache.insert(reg, val);
        }

        fn update_cfa(
            &mut self,
            rule: &CfaRule<EndianSlice<LittleEndian>>,
            core: &mut Core,
        ) -> Result</* cfa_changed: */ bool, anyhow::Error> {
            debug!("Registers::update_cfg(self={:?}, rule={:?})", self, rule);

            match rule {
                CfaRule::RegisterAndOffset { register, offset } => {
                    let cfa = (i64::from(self.get(gimli2cortex(register), core)?) + offset) as u32;
                    let ok = self.cache.get(&Register::SP) != Some(&cfa);
                    self.cache.insert(Register::SP, cfa);
                    Ok(ok)
                }

                CfaRule::Expression(_) => unimplemented!("CfaRule::Expression"),
            }
        }

        fn update(
            &mut self,
            reg: &gimli::Register,
            rule: &RegisterRule<EndianSlice<LittleEndian>>,
            core: &mut Core,
        ) -> Result<(), anyhow::Error> {
            let reg = gimli2cortex(reg);
            debug!(
                "Registers::update(self={:?}, reg={:?}, rule={:?})",
                self, reg, rule
            );

            match rule {
                RegisterRule::Undefined => unreachable!(),

                RegisterRule::Offset(offset) => {
                    let cfa = self.get(Register::SP, core)?;
                    let addr = (i64::from(cfa) + offset) as u32;
                    self.cache.insert(reg, core.read_word_32(addr)?);
                }

                _ => unimplemented!(),
            }

            Ok(())
        }
    }

    use cortex_m::Register;

    // statically linked binary -- there are no relative addresses
    let bases = &BaseAddresses::default();
    let ctx = &mut UninitializedUnwindContext::new();

    println!("stack backtrace:");
    let mut frame = 0;
    let mut registers = Registers::new(lr, sp);
    loop {
        println!(
            "{:>4}: {:#010x} - {}",
            frame,
            pc,
            rustc_demangle::demangle(
                range_names
                    .binary_search_by(|rn| if rn.0.contains(&u64::from(pc)) {
                        cmp::Ordering::Equal
                    } else if u64::from(pc) < rn.0.start {
                        cmp::Ordering::Greater
                    } else {
                        cmp::Ordering::Less
                    })
                    .map(|idx| &*range_names[idx].1)
                    .unwrap_or("<unknown>")
            )
        );

        let fde = debug_frame.fde_for_address(bases, pc.into(), DebugFrame::cie_from_offset)?;
        let uwt_row = fde.unwind_info_for_address(debug_frame, bases, ctx, pc.into())?;

        let cfa_changed = registers.update_cfa(uwt_row.cfa(), core)?;

        for (reg, rule) in uwt_row.registers() {
            registers.update(reg, rule, core)?;
        }

        let lr = registers.get(Register::LR, core)?;
        if lr == LR_END {
            break;
        }

        if !cfa_changed && lr == pc {
            println!("error: the stack appears to be corrupted beyond this point");
            return Ok(());
        }

        if lr > 0xffff_fff0 {
            println!("      <exception entry>");

            let sp = registers.get(Register::SP, core)?;
            let stacked = Stacked::read(core, sp)?;

            // XXX insert other registers?
            registers.insert(Register::LR, stacked.lr);
            // adjust the stack pointer for stacked registers
            registers.insert(Register::SP, sp + mem::size_of::<Stacked>() as u32);
            pc = stacked.pc;
        } else {
            if lr & 1 == 0 {
                bail!("bug? LR ({:#010x}) didn't have the Thumb bit set", lr)
            }
            pc = lr & !1;
        }

        frame += 1;
    }

    Ok(())
}

#[derive(Debug)]
struct Stacked {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: u32,
    xpsr: u32,
}

impl Stacked {
    fn read(core: &mut Core, sp: u32) -> Result<Self, anyhow::Error> {
        let mut registers = [0; 8];
        core.read_32(sp, &mut registers)?;

        Ok(Stacked {
            r0: registers[0],
            r1: registers[1],
            r2: registers[2],
            r3: registers[3],
            r12: registers[4],
            lr: registers[5],
            pc: registers[6],
            xpsr: registers[7],
        })
    }
}

fn read_word_16(core: &mut Core, addr: u32) -> Result<u16, anyhow::Error> {
    if addr % 4 == 0 {
        Ok(core.read_word_32(addr)? as u16)
    } else if addr % 4 == 2 {
        Ok((core.read_word_32(addr - 2)? >> 16) as u16)
    } else {
        unreachable!()
    }
}
