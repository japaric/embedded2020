MEMORY
{
  /* NOTE RAM is split as follows */
  /* - 8 AHB slaves, each connected to a 2x4 KB RAM sections */
  /* - the 9th AHB slave is connected to 6x32 KB RAM sections */
  /* NOTE all RAM is aliased at address 0x0080_0000 for use as Code RAM */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

ENTRY(Reset);

/* "End-align" all sections to the end (highest address) of the RAM memory
   region then place the stack *before* (lower address) those sections. We'll
   end with something like this (excuse the poor ASCII art):

   0x2000_0000                 *                                   0x2004_0000
   |                   <-stack | .text .rodata .bss .data .vectors |

   `|` denotes the physical boundaries of RAM
   `*` is the address of the `__stack_top__` symbol
*/

SECTIONS
{
  .vectors ORIGIN(RAM) + LENGTH(RAM) - SIZEOF(.vectors) :
  {
    LONG(__stack_top__);
    LONG(Reset);
    LONG(NMI);
    LONG(HardFault);
    LONG(MemManage);
    LONG(BusFault);
    LONG(UsageFault);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(SVCall);
    LONG(DebugMonitor);
    LONG(Reserved);
    LONG(PendSV);
    LONG(SysTick);

    /* TODO give names to the 37 device-specific interrupts */
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(DefaultHandler);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
    LONG(Reserved);
  } > RAM

  /* NOTE `ALIGN(4)` is used to 4-byte align the start and end of all sections */
  .data ADDR(.vectors) - SIZEOF(.data) :
  {
    . = ALIGN(4);
    *(.data .data.*);
    . = ALIGN(4);
  } > RAM

  .bss ADDR(.data) - SIZEOF(.bss) :
  {
    . = ALIGN(4);
    *(.bss .bss.*);
    . = ALIGN(4);
  } > RAM

  .rodata ADDR(.bss) - SIZEOF(.rodata) :
  {
    . = ALIGN(4);
    *(.rodata .rodata.*);
    . = ALIGN(4);
  } > RAM

  .text ADDR(.rodata) - SIZEOF(.text) :
  {
    . = ALIGN(4);
    *(.text .text.*);
    . = ALIGN(4);
  } > RAM

  __stack_top__ = ADDR(.text);

  /* ## Discarded sections */
  /DISCARD/ :
  {
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}

/* Weak exceptions */
Reserved = 0;
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
