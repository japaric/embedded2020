MEMORY
{
  /* NOTE RAM is split as follows */
  /* - 8 AHB slaves, each connected to a 2x4 KB RAM sections */
  /* - the 9th AHB slave is connected to 6x32 KB RAM sections */
  /* NOTE all RAM is aliased at address 0x0080_0000 for use as Code RAM */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

ENTRY(Reset);
PROVIDE(__stack_top__ = ORIGIN(RAM) + LENGTH(RAM));
PROVIDE(__ram_start__ = ORIGIN(RAM));

SECTIONS
{
  /* stack located here */

  .uninit __ram_start__ (NOLOAD) :
  {
    *(.uninit.*);
    . = ALIGN(4);
  } > RAM

  .bss ADDR(.uninit) + SIZEOF(.uninit) (NOLOAD) :
  {
    _sbss = .;
    *(.bss .bss.*);
    . = ALIGN(4);
    _ebss = .;
  } > RAM

  .data ADDR(.bss) + SIZEOF(.bss) :
  {
    *(.data .data.*);
    . = ALIGN(4);
  } > RAM

  .init ADDR(.data) + SIZEOF(.data) :
  {
    _sinit = .;
    KEEP(*(.init.*));
    /* no ALIGN because this section's size is always multiple of 4 bytes */
    _einit = .;
  } > RAM

  .rodata ADDR(.init) + SIZEOF(.init) :
  {
    *(.rodata .rodata.*);
    . = ALIGN(4);
  } > RAM

  .text ADDR(.rodata) + SIZEOF(.rodata) :
  {
    *(.text .text.*);
    /* `.vectors` alignment requirement given the size of the vector table */
    . = ALIGN(256);
  } > RAM

  .vectors ADDR(.text) + SIZEOF(.text) :
  {
    KEEP(*(.vectors));
  } > RAM

  .binfmt (INFO) :
  {
    *(.binfmt.*);
  }

  /* ## Discarded sections */
  /DISCARD/ :
  {
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}

ASSERT(SIZEOF(.binfmt) < 16384, "SIZEOF(.binfmt) must not exceed 16383 bytes");
ASSERT(_sinit % 4 == 0 && _einit % 4 == 0, "`.init` section is not 4-byte aligned");

INCLUDE interrupts.x
