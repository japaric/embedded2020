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

    let mut ir_regs = vec![];

    for cluster in regs {
        match cluster {
            svd::RegisterCluster::Register(r) => {
                register_(r, None, &[defaults], &mut ir_regs)
            }

            svd::RegisterCluster::Cluster(cluster) => match cluster {
                svd::Cluster::Single(info) => {
                    for child in &info.children {
                        match child {
                            svd::RegisterCluster::Register(r) => register_(
                                r,
                                Some(info),
                                &[&info.default_register_properties, defaults],
                                &mut ir_regs,
                            ),

                            svd::RegisterCluster::Cluster(..) => {
                                unimplemented!()
                            }
                        }
                    }
                }

                svd::Cluster::Array(ci, dim) => {
                    assert!(dim.dim_index.is_none(), "unimplemented");
                    assert!(ci.name.contains("[%s]"), "unimplemented");

                    let template = &ci.name;
                    let offset = ci.address_offset;

                    for i in 0..dim.dim {
                        // FIXME too lazy to do ownership correctly right now
                        let ci: &'static mut _ =
                            Box::leak(Box::new(ci.clone()));

                        ci.name = template.replace("[%s]", &i.to_string());
                        ci.address_offset = offset + i * dim.dim_increment;

                        for child in &ci.children {
                            match child {
                                svd::RegisterCluster::Register(ri) => {
                                    ir_regs.push(translate::register(
                                        ri,
                                        Some(ci),
                                        &[
                                            &ci.default_register_properties,
                                            defaults,
                                        ],
                                    ));
                                }

                                svd::RegisterCluster::Cluster(..) => {
                                    unimplemented!()
                                }
                            }
                        }
                    }
                }
            },
        }
    }

    ir::Peripheral {
        name: p.name.as_str().into(),
        description: p.description.as_ref().map(|s| s.into()),
        instances: ir::Instances::Single {
            base_address: u64::from(p.base_address),
        },
        registers: ir_regs,
    }
}

fn register_<'a>(
    r: &'a svd::Register,
    ci: Option<&svd::ClusterInfo>,
    defaults: &[&svd::RegisterProperties],
    ir_regs: &mut Vec<ir::Register<'a>>,
) {
    match r {
        svd::Register::Single(ri) => {
            ir_regs.push(translate::register(ri, ci, defaults));
        }

        svd::Register::Array(ri, dim) => {
            assert!(dim.dim_index.is_none(), "unimplemented");
            assert!(ri.name.contains("[%s]"), "unimplemented");

            let template = &ri.name;
            let offset = ri.address_offset;

            for i in 0..dim.dim {
                // FIXME too lazy to do ownership correctly right now
                let mut ri: &'static mut _ = Box::leak(Box::new(ri.clone()));

                ri.name = template.replace("[%s]", &i.to_string());
                ri.address_offset = offset + i * dim.dim_increment;

                ir_regs.push(translate::register(ri, ci, defaults));
            }
        }
    }
}

pub fn register<'a>(
    r: &'a svd::RegisterInfo,
    cluster: Option<&svd::ClusterInfo>,
    defaults: &[&svd::RegisterProperties],
) -> ir::Register<'a> {
    let (r_fields, w_fields) = r
        .fields
        .as_ref()
        .map(|f| translate::fields(f, r))
        .unwrap_or((vec![], vec![]));

    let (name, offset) = if let Some(cluster) = cluster {
        (
            format!("{}_{}", cluster.name, r.name).into(),
            cluster.address_offset + r.address_offset,
        )
    } else {
        (r.name.as_str().into(), r.address_offset)
    };

    ir::Register {
        access: r
            .access
            .or_else(|| {
                defaults.iter().filter_map(|default| default.access).next()
            })
            .map(translate::access)
            .expect("unimplemented"),
        description: r.description.as_ref().map(|s| s.as_str().into()),
        name,
        r_fields,
        w_fields,
        offset: u64::from(offset),
        width: r
            .size
            .or_else(|| {
                defaults.iter().filter_map(|default| default.size).next()
            })
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
