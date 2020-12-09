MEMORY
{
  /* Bootloader is split in 2 parts: the first part lives in the range
  0..0x1000; the second part lives at the end of the 1 MB Flash. The range
  selected here collides with neither */
  FLASH : ORIGIN = 0x1000, LENGTH = 0x7f000

/* The bootloader uses the first 8 bytes of RAM to preserve state so don't
touch them */
RAM   : ORIGIN = 0x20000008, LENGTH = 0x3fff8
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
