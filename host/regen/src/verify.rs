use anyhow::{bail, Context};

use crate::{
    fmt::Hex,
    ir::{Bitfield, Device, Instances, Peripheral, Register},
};

impl Device<'_> {
    pub fn verify(&self) -> Result<(), anyhow::Error> {
        for peripheral in &self.peripherals {
            peripheral.verify()?;
        }

        Ok(())
    }
}

impl Peripheral<'_> {
    fn verify(&self) -> Result<(), anyhow::Error> {
        if self.name.is_empty() {
            bail!("unnamed peripheral");
        }

        (|| {
            match &self.instances {
                Instances::Many { instances } => {
                    let n = instances.len();
                    if n < 2 {
                        bail!(
                            "specified `Instances::Many` but it contains less than 2 ({}) ",
                            n
                        )
                    }
                }

                Instances::Single { .. } => {}
            }

            for reg in &self.registers {
                reg.verify()?;

                // TODO check for register overlap
            }

            Ok(())
        })()
        .context(format!("while verifying peripheral {}", self.name))?;

        Ok(())
    }
}

impl Register<'_> {
    fn verify(&self) -> Result<(), anyhow::Error> {
        if self.name.is_empty() {
            bail!("unnamed register with offset {}", Hex(self.offset));
        }

        (|| {
            let reg_width = self.width.bits();
            for field in self.r_fields.iter().chain(&self.w_fields) {
                field.verify()?;

                if field.width + field.offset > reg_width {
                    bail!(
                        "bitfield {} (offset: {}, width: {}) exceeds register width ({})",
                        field.name,
                        field.offset,
                        field.width,
                        reg_width,
                    )
                }
            }

            fn check_for_overlap(fields: &[Bitfield<'_>]) -> Result<(), anyhow::Error> {
                let mut used: u64 = 0;
                for field in fields {
                    let mask = ((1 << field.width) - 1) << field.offset;

                    if used & mask != 0 {
                        bail!("bitfield {} overlaps with other bitfields", field.name);
                    }

                    used |= mask;
                }

                Ok(())
            }

            check_for_overlap(&self.r_fields)?;
            check_for_overlap(&self.w_fields)?;

            Ok(())
        })()
        .context(format!("while verifying register {}", self.name))?;

        Ok(())
    }
}

impl Bitfield<'_> {
    fn verify(&self) -> Result<(), anyhow::Error> {
        if self.name.is_empty() {
            bail!("unnamed bitfield at offset {}", Hex(self.offset));
        }

        if self.width == 0 {
            bail!("bitfield {} has a width of 0 bits", self.name);
        }

        Ok(())
    }
}
