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
  .text __ram_start__ :
  {
    *(.text .text.*);
    . = ALIGN(4);
  } > RAM

  .rodata ADDR(.text) + SIZEOF(.text) :
  {
    *(.rodata .rodata.*);
    . = ALIGN(4);
  } > RAM

  .data ADDR(.rodata) + SIZEOF(.rodata) :
  {
    *(.data .data.*);
    . = ALIGN(4);
  } > RAM

  .bss ADDR(.data) + SIZEOF(.data) :
  {
    *(.bss .bss.*);
    . = ALIGN(4);
  } > RAM

  .uninit ADDR(.bss) + SIZEOF(.bss) :
  {
    *(.uninit.*);
    . = ALIGN(256);
  } > RAM

  .vectors ADDR(.uninit) + SIZEOF(.uninit) :
  {
    /* alignment requirement given the size of the vector table */
    . = ALIGN(256);
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

ASSERT(SIZEOF(.log) <= 256, "SIZEOF(.log) must not exceed 256 bytes");

/* Weak exceptions */
Reserved = 0;
EXTERN(VECTORS);
EXTERN(DefaultHandler);
PROVIDE(NMI = DefaultHandler);
PROVIDE(HardFault = DefaultHandler);
PROVIDE(MemManage = DefaultHandler);
PROVIDE(BusFault = DefaultHandler);
PROVIDE(UsageFault = DefaultHandler);
PROVIDE(SVCall = DefaultHandler);
PROVIDE(DebugMonitor = DefaultHandler);
PROVIDE(PendSV = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);

/* TODO weak interrupts */
