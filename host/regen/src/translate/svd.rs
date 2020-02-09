use core::convert::TryInto;

use svd_parser as svd;

use crate::{ir, translate::svd as translate};

pub fn device<'a>(d: &'a svd::Device, whitelist: &[&str]) -> ir::Device<'a> {
    let defaults = &d.default_register_properties;
    let mut peripherals = vec![];
    for periph in &d.peripherals {
        if whitelist.contains(&&*periph.name) {
            // skip peripheral with no registers
            if let Some(regs) = periph.registers.as_ref() {
                peripherals
                    .push(translate::peripheral(&periph, regs, defaults));
            }
        }
    }

    ir::Device {
        extra_docs: None,
        name: d.name.as_str().into(),
        peripherals,
    }
}

pub fn peripheral<'a>(
    p: &'a svd::Peripheral,
    regs: &'a [svd::RegisterCluster],
    defaults: &svd::RegisterProperties,
) -> ir::Peripheral<'a> {
    assert!(p.derived_from.is_none());

    ir::Peripheral {
        name: p.name.as_str().into(),
        description: p.description.as_ref().map(|s| s.into()),
        instances: ir::Instances::Single {
            base_address: u64::from(p.base_address),
        },
        registers: regs
            .iter()
            .filter_map(|cluster| match cluster {
                svd::RegisterCluster::Register(r) => match r {
                    svd::Register::Single(ri) => {
                        Some(translate::register(ri, defaults))
                    }
                    // TODO implement
                    svd::Register::Array(..) => None,
                },
                svd::RegisterCluster::Cluster(..) => unimplemented!(),
            })
            .collect(),
    }
}

pub fn register<'a>(
    r: &'a svd::RegisterInfo,
    defaults: &svd::RegisterProperties,
) -> ir::Register<'a> {
    let (r_fields, w_fields) = r
        .fields
        .as_ref()
        .map(|f| translate::fields(f, r))
        .unwrap_or((vec![], vec![]));

    ir::Register {
        access: r
            .access
            .or(defaults.access)
            .map(translate::access)
            .expect("unimplemented"),
        description: r.description.as_ref().map(|s| s.as_str().into()),
        name: r.name.as_str().into(),
        r_fields,
        w_fields,
        offset: u64::from(r.address_offset),
        width: r
            .size
            .or(defaults.size)
            .map(translate::register_size)
            .expect("unimplemented"),
    }
}

pub fn access(access: svd::Access) -> ir::Access {
    match access {
        svd::Access::ReadOnly => ir::Access::ReadOnly,
        svd::Access::WriteOnly => ir::Access::WriteOnly {
            unsafe_write: false,
        },
        svd::Access::ReadWrite => ir::Access::ReadWrite {
            unsafe_write: false,
        },
        _ => unimplemented!("{:?}", access),
    }
}

pub fn fields<'a>(
    fields: &'a [svd::Field],
    reg: &'a svd::RegisterInfo,
) -> (Vec<ir::Bitfield<'a>>, Vec<ir::Bitfield<'a>>) {
    let mut r_fields = vec![];
    let mut w_fields = vec![];

    for field in fields {
        match field {
            svd::Field::Single(fi) => {
                let (offset, width) = translate::bit_range(fi.bit_range);
                let bf = ir::Bitfield {
                    description: fi
                        .description
                        .as_ref()
                        .map(|s| s.as_str().into()),
                    name: fi.name.as_str().into(),
                    offset,
                    width,
                };

                match fi.access.or(reg.access).expect("unreachable") {
                    svd::Access::ReadOnly => r_fields.push(bf),
                    svd::Access::WriteOnly => w_fields.push(bf),
                    svd::Access::ReadWrite => {
                        r_fields.push(bf.clone());
                        w_fields.push(bf);
                    }
                    access => unimplemented!("{:?}", access),
                }
            }
            svd::Field::Array(..) => unimplemented!(),
        }
    }

    (r_fields, w_fields)
}

fn register_size(size: u32) -> ir::Width {
    match size {
        8 => ir::Width::U8,
        16 => ir::Width::U16,
        32 => ir::Width::U32,
        64 => ir::Width::U64,
        _ => unreachable!(),
    }
}

fn bit_range(br: svd::BitRange) -> (u8, u8) {
    (
        br.offset.try_into().expect("unreachable"),
        br.width.try_into().expect("unreachable"),
    )
}
