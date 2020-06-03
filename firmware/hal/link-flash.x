MEMORY
{
  /* NOTE RAM is split as follows */
  /* - 8 AHB slaves, each connected to a 2x4 KB RAM sections */
  /* - the 9th AHB slave is connected to 6x32 KB RAM sections */
  /* NOTE all RAM is aliased at address 0x0080_0000 for use as Code RAM */
  /* FLASH : ORIGIN = 0x1000, LENGTH = 0x7F000 */
  FLASH : ORIGIN = 0, LENGTH = 0x7F000
  RAM : ORIGIN = 0x20000008, LENGTH = 256K
}

ENTRY(Reset);
PROVIDE(__stack_top__ = ORIGIN(RAM) + LENGTH(RAM));
PROVIDE(__ram_start__ = ORIGIN(RAM));

SECTIONS
{
  .vectors :
  {
    KEEP(*(.vectors));
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*);
    . = ALIGN(4);
  } > FLASH

  .text :
  {
    *(.text .text.*);
  } > FLASH

  .init :
  {
    _sinit = .;
    KEEP(*(.init.*));
    /* no ALIGN because this section's size is always multiple of 4 bytes */
    _einit = .;
  } > FLASH

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
    _sdata = .;
    *(.data .data.*);
    . = ALIGN(4);
    _edata = .;
  } > RAM AT>FLASH

  _sidata = LOADADDR(.data);

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
ASSERT(ADDR(.vectors) == ORIGIN(FLASH), "vector table has been misplaced");

INCLUDE interrupts.x
