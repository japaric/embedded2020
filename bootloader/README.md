# `bootloader**

> A super simple and small "bootloader" for the nRF52840 that executes a program
> *previously* loaded into RAM

Yes, this bootloader doesn't actually provide a mechanism to load programs into
memory. You'll need a a tool like `semidap` for that step.

# Flashing

*Requires*: git version of OpenOCD (version 0.10 doesn't support flashing the
nRF52840 chip) and `arm-none-eabi-gdb` (if you are using a multi-arch GDB with a
different name then change the name of the command in `.cargo/config`)

To flash this bootloader into the board run these commands:

``` console
$ # from within the bootloader directory

$ # on one terminal
$ openocd

$ # on another terminal
$ cargo +nightly r --release
```

# How it works

The bootloader excepts a vector table with 64 entries (256 bytes in size) at the
*end* of RAM, so *end-aligned* to the `0x2004_0000` boundary. On power-on the
bootloader will verify the correctness of the table by:

- checking that the initial Stack Pointer value is 8-byte aligned
- all reserved entries (`7..11` (end-exclusive), `13` and `(16+37)..`) are set
  to the value of `0`
- all other entries, the real vectors, are odd addresses (Thumb bit is set)

If the table is deemed correct the bootloader will set the Stack Pointer to the
value indicated in the RAM vector table (first entry), set the `VTOR` register
to the start of the RAM vector table and then jump into the second entry of the
RAM vector table, the `Reset` handler.

If the table is not deemed correct (e.g. it has random contents after POR) then
the green LED will be turned on to indicate that bootloader is ready to receive
a payload. After flashing a program in RAM you'll have to trigger a reset of the
processor (using SYSRESETREQ, for example; `semidap` does this) to make the
bootloader jump into the loaded program. Note that power cycling will erase the
contents of the RAM!

If the bootloader runs into a unrecoverable error, like an exception, the red
LED will be turned on.

The bootloader is configured to not contain any static variable that may reside
in RAM and to generate a `HardFault` exception if it uses *any* stack memory.
Therefore the bootloader uses zero bytes of RAM so the loaded program can span
and use the whole RAM region.
