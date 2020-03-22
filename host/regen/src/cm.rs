//! # References
//!
//! - (TRM) Cortex-M4 r0p0 Technical Reference Manual (ARM DDI 0439B)
//! - (ARM) ARMv7-M Architecture Reference Manual (ARM DDI 0403E.b)

use crate::ir::{Access, Bitfield, Device, Instances, Peripheral, Register, Width};

pub fn device() -> Device<'static> {
    Device {
        extra_docs: Some(
            "# References
- ARMv7-M Architecture Reference Manual (ARM DDI 0403E.b)"
                .into(),
        ),
        name: "Cortex-M".into(),
        peripherals: peripherals(),
    }
}

fn peripherals() -> Vec<Peripheral<'static>> {
    vec![
        Peripheral {
            description: Some("Debug Control Block".into()),
            instances: Instances::Single {
                base_address: 0xE000_EDF0,
            },
            name: "DCB".into(),
            registers: vec![
                {
                    let fields = vec![
                        Bitfield {
                            description: None,
                            name: "C_DEBUGEN".into(),
                            offset: 0,
                            width: 1,
                        },
                        Bitfield {
                            description: None,
                            name: "C_HALT".into(),
                            offset: 1,
                            width: 1,
                        },
                        Bitfield {
                            description: None,
                            name: "C_STEP".into(),
                            offset: 2,
                            width: 1,
                        },
                        Bitfield {
                            description: None,
                            name: "C_MASKINTS".into(),
                            offset: 3,
                            width: 1,
                        },
                        Bitfield {
                            description: None,
                            name: "C_SNAPSTALL".into(),
                            offset: 5,
                            width: 1,
                        },
                    ];

                    let mut r_fields = fields.clone();

                    r_fields.push(Bitfield {
                        description: None,
                        name: "S_REGRDY".into(),
                        offset: 16,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "S_HALT".into(),
                        offset: 17,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "S_SLEEP".into(),
                        offset: 18,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "S_LOCKUP".into(),
                        offset: 19,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "S_RETIRE_ST".into(),
                        offset: 24,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "S_RESET_ST".into(),
                        offset: 25,
                        width: 1,
                    });

                    let mut w_fields = fields;
                    w_fields.push(Bitfield {
                        description: None,
                        name: "DBGKEY".into(),
                        offset: 16,
                        width: 16,
                    });

                    // section C1.6.2 of (ARM)
                    Register {
                        access: Access::ReadWrite {
                            unsafe_write: false,
                        },
                        description: Some("Debug Halting Control and Status Register".into()),
                        name: "DHCSR".into(),
                        offset: 0x00,
                        r_fields,
                        w_fields,
                        width: Width::U32,
                    }
                },
                {
                    let mut w_fields = vec![];
                    w_fields.push(Bitfield {
                        description: None,
                        name: "REGSEL".into(),
                        offset: 0,
                        width: 7,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "REGWnR".into(),
                        offset: 16,
                        width: 1,
                    });

                    Register {
                        access: Access::WriteOnly {
                            unsafe_write: false,
                        },
                        description: Some("Debug Core Register Selector Register".into()),
                        name: "DCRSR".into(),
                        offset: 0x04,
                        r_fields: vec![],
                        w_fields,
                        width: Width::U32,
                    }
                },
                {
                    Register {
                        access: Access::ReadWrite { unsafe_write: true },
                        description: Some("Debug Core Register Data Register".into()),
                        name: "DCRDR".into(),
                        offset: 0x08,
                        r_fields: vec![],
                        w_fields: vec![],
                        width: Width::U32,
                    }
                },
                {
                    let mut fields = vec![];
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_CORERESET".into(),
                        offset: 0,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_MMERR".into(),
                        offset: 4,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_NOCPERR".into(),
                        offset: 5,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_CHKERR".into(),
                        offset: 6,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_STATERR".into(),
                        offset: 7,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_BUSERR".into(),
                        offset: 8,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_INTERR".into(),
                        offset: 9,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "VC_HARDERR".into(),
                        offset: 10,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "MON_EN".into(),
                        offset: 16,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "MON_PEND".into(),
                        offset: 17,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "MON_STEP".into(),
                        offset: 18,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "MON_REQ".into(),
                        offset: 19,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: Some(
                            "Enables the DWT and ITM:\n0: DWT and ITM are disabled.\n1: DWT and ITM are enabled."
                                .into(),
                        ),
                        name: "TRCENA".into(),
                        offset: 24,
                        width: 1,
                    });

                    // section C.1.6.5 of (ARM)
                    Register {
                        access: Access::ReadWrite {
                            unsafe_write: false,
                        },
                        description: Some("Debug Exception and Monitor Control Register".into()),
                        name: "DEMCR".into(),
                        offset: 0x0c,
                        r_fields: fields.clone(),
                        w_fields: fields,
                        width: Width::U32,
                    }
                },
            ],
        },
        Peripheral {
            description: Some("Data Watchpoint and Trace".into()),
            instances: Instances::Single {
                base_address: 0xE000_1000,
            },
            name: "DWT".into(),
            registers: vec![
                {
                    let mut w_fields = vec![];
                    w_fields.push(Bitfield {
                        description: Some(
                            "Enables the cycle counter.\n0: Counter disabled.\n1: Counter enabled."
                                .into(),
                        ),
                        name: "CYCCNTENA".into(),
                        offset: 0,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "POSTPRESET".into(),
                        offset: 1,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "POSTINIT".into(),
                        offset: 5,
                        width: 4,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "CYCTAP".into(),
                        offset: 9,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "SYNCTAP".into(),
                        offset: 10,
                        width: 2,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "PCSAMPLENA".into(),
                        offset: 12,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "EXCTRCENA".into(),
                        offset: 16,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "CPIEVTENA".into(),
                        offset: 17,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "EXCEVTENA".into(),
                        offset: 18,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "SLEEPEVTENA".into(),
                        offset: 19,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "LSUEVTENA".into(),
                        offset: 20,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "FOLDEVTENA".into(),
                        offset: 21,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "CYCEVTENA".into(),
                        offset: 22,
                        width: 1,
                    });

                    let mut r_fields = w_fields.clone();
                    r_fields.push(Bitfield {
                        description: None,
                        name: "NOPRFCNT".into(),
                        offset: 24,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "NOCYCCNT".into(),
                        offset: 25,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "NOEXTTRIG".into(),
                        offset: 26,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "NOTRCPKT".into(),
                        offset: 27,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "NUMCOMP".into(),
                        offset: 28,
                        width: 4,
                    });

                    // section C1.8.7 of (ARM)
                    Register {
                        access: Access::ReadWrite {
                            unsafe_write: false,
                        },
                        description: Some("Control register".into()),
                        name: "CTRL".into(),
                        offset: 0x0,
                        r_fields,
                        w_fields,
                        width: Width::U32,
                    }
                },
                // section C1.8.8 of (ARM)
                Register {
                    access: Access::ReadWrite {
                        unsafe_write: false,
                    },
                    description: Some("Cycle Count register".into()),
                    name: "CYCCNT".into(),
                    offset: 0x4,
                    r_fields: vec![],
                    w_fields: vec![],
                    width: Width::U32,
                },
            ],
        },
        Peripheral {
            description: Some("Nested Vector Interrupt Controller".into()),
            instances: Instances::Single {
                base_address: 0xE000_E100,
            },
            name: "NVIC".into(),
            registers: vec![
                // NOTE(unsafe_write) enabling interrupts can break critical section
                Register {
                    access: Access::ReadWrite { unsafe_write: true },
                    description: Some("Interrupt Set-Enable Register 0".into()),
                    name: "ISER0".into(),
                    offset: 0x0,
                    r_fields: vec![],
                    w_fields: vec![],
                    width: Width::U32,
                },
                Register {
                    access: Access::ReadWrite { unsafe_write: true },
                    description: Some("Interrupt Set-Enable Register 1".into()),
                    name: "ISER1".into(),
                    offset: 0x4,
                    r_fields: vec![],
                    w_fields: vec![],
                    width: Width::U32,
                },
                Register {
                    access: Access::ReadWrite {
                        unsafe_write: false,
                    },
                    description: Some("Interrupt Clear-Enable Register 0".into()),
                    name: "ICER0".into(),
                    offset: 0x80,
                    r_fields: vec![],
                    w_fields: vec![],
                    width: Width::U32,
                },
                Register {
                    access: Access::ReadWrite {
                        unsafe_write: false,
                    },
                    description: Some("Interrupt Clear-Enable Register 1".into()),
                    name: "ICER1".into(),
                    offset: 0x84,
                    r_fields: vec![],
                    w_fields: vec![],
                    width: Width::U32,
                },
            ],
        },
        Peripheral {
            description: Some("System Control Block".into()),
            instances: Instances::Single {
                base_address: 0xE000_ED00,
            },
            name: "SCB".into(),
            registers: vec![
                {
                    let mut r_fields = vec![];
                    r_fields.push(Bitfield {
                        description: None,
                        name: "REVISION".into(),
                        offset: 0,
                        width: 4,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "PARTNO".into(),
                        offset: 4,
                        width: 12,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "ARCHITECTURE".into(),
                        offset: 16,
                        width: 4,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "VARIANT".into(),
                        offset: 20,
                        width: 4,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "IMPLEMENTER".into(),
                        offset: 24,
                        width: 8,
                    });

                    // section B3.2.3 of (ARM)
                    Register {
                        access: Access::ReadOnly,
                        description: Some("CPUID Base register".into()),
                        name: "CPUID".into(),
                        offset: 0x0,
                        r_fields,
                        w_fields: vec![],
                        width: Width::U32,
                    }
                },
                {
                    let mut fields = vec![];
                    fields.push(Bitfield {
                        description: None,
                        name: "PENDSTSET".into(),
                        offset: 26,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "PENDSVSET".into(),
                        offset: 28,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "NMIPENDSET".into(),
                        offset: 31,
                        width: 1,
                    });

                    let mut w_fields = fields.clone();

                    w_fields.push(Bitfield {
                        description: None,
                        name: "PENDSTCLR".into(),
                        offset: 25,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "PENDSVCLR".into(),
                        offset: 27,
                        width: 1,
                    });

                    let mut r_fields = fields;

                    r_fields.push(Bitfield {
                    description: Some("The vector table index of the exception currently being executed.\n0: Thread mode\n!0: Exception context".into()),
                    name: "VECTACTIVE".into(),
                    offset: 0,
                    width: 9,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "RETTOBASE".into(),
                        offset: 11,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "VECTPENDING".into(),
                        offset: 12,
                        width: 9,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "ISRPENDING".into(),
                        offset: 22,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "ISRPREEMPT".into(),
                        offset: 23,
                        width: 1,
                    });

                    // section B3.2.4 of (ARM)
                    Register {
                        access: Access::ReadWrite {
                            unsafe_write: false,
                        },
                        description: Some("Interrupt Control and State Register".into()),
                        name: "ICSR".into(),
                        offset: 0x4,
                        r_fields,
                        w_fields,
                        width: Width::U32,
                    }
                },
                {
                    let fields = vec![Bitfield {
                        description: None,
                        name: "TBLOFF".into(),
                        offset: 7,
                        width: 25,
                    }];

                    // section B3.2.5 of (ARM)
                    Register {
                        access: Access::ReadWrite { unsafe_write: true },
                        description: Some("Vector Table Offset Register".into()),
                        name: "VTOR".into(),
                        offset: 0x8,
                        r_fields: fields.clone(),
                        w_fields: fields,
                        width: Width::U32,
                    }
                },
                {
                    let mut fields = vec![];
                    fields.push(Bitfield {
                        description: None,
                        name: "SYSRESETREQ".into(),
                        offset: 2,
                        width: 1,
                    });
                    fields.push(Bitfield {
                        description: None,
                        name: "PRIGROUP".into(),
                        offset: 8,
                        width: 3,
                    });

                    let mut r_fields = fields.clone();
                    r_fields.push(Bitfield {
                        description: None,
                        name: "ENDIANNESS".into(),
                        offset: 15,
                        width: 1,
                    });
                    r_fields.push(Bitfield {
                        description: None,
                        name: "VECTKEYSTAT".into(),
                        offset: 16,
                        width: 16,
                    });

                    let mut w_fields = fields;
                    w_fields.push(Bitfield {
                        description: None,
                        name: "VECTRESET".into(),
                        offset: 0,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "VECTCLRACTIVE".into(),
                        offset: 1,
                        width: 1,
                    });
                    w_fields.push(Bitfield {
                        description: None,
                        name: "VECTKEY".into(),
                        offset: 16,
                        width: 16,
                    });

                    Register {
                        access: Access::ReadWrite {
                            unsafe_write: false,
                        },
                        description: Some(
                            "Application Interrupt and Reset Control Register".into(),
                        ),
                        name: "AIRCR".into(),
                        offset: 0xc,
                        r_fields,
                        w_fields,
                        width: Width::U32,
                    }
                },
            ],
        },
    ]
}
