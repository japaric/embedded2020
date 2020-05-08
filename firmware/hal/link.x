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

/* Weak interrupts */
PROVIDE(POWER_CLOCK = DefaultHandler);
PROVIDE(POWER = DefaultHandler);
PROVIDE(CLOCK = DefaultHandler);
PROVIDE(RADIO = DefaultHandler);
PROVIDE(UARTE0_UART0 = DefaultHandler);
PROVIDE(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 = DefaultHandler);
PROVIDE(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 = DefaultHandler);
PROVIDE(NFCT = DefaultHandler);
PROVIDE(GPIOTE = DefaultHandler);
PROVIDE(SAADC = DefaultHandler);
PROVIDE(TIMER0 = DefaultHandler);
PROVIDE(TIMER1 = DefaultHandler);
PROVIDE(TIMER2 = DefaultHandler);
PROVIDE(RTC0 = DefaultHandler);
PROVIDE(TEMP = DefaultHandler);
PROVIDE(RNG = DefaultHandler);
PROVIDE(ECB = DefaultHandler);
PROVIDE(CCM_AAR = DefaultHandler);
PROVIDE(WDT = DefaultHandler);
PROVIDE(RTC1 = DefaultHandler);
PROVIDE(QDEC = DefaultHandler);
PROVIDE(COMP_LPCOMP = DefaultHandler);
PROVIDE(SWI0_EGU0 = DefaultHandler);
PROVIDE(SWI1_EGU1 = DefaultHandler);
PROVIDE(SWI2_EGU2 = DefaultHandler);
PROVIDE(SWI3_EGU3 = DefaultHandler);
PROVIDE(SWI4_EGU4 = DefaultHandler);
PROVIDE(SWI5_EGU5 = DefaultHandler);
PROVIDE(TIMER3 = DefaultHandler);
PROVIDE(TIMER4 = DefaultHandler);
PROVIDE(PWM0 = DefaultHandler);
PROVIDE(PDM = DefaultHandler);
PROVIDE(MWU = DefaultHandler);
PROVIDE(PWM1 = DefaultHandler);
PROVIDE(PWM2 = DefaultHandler);
PROVIDE(SPIM2_SPIS2_SPI2 = DefaultHandler);
PROVIDE(RTC2 = DefaultHandler);
PROVIDE(I2S = DefaultHandler);
PROVIDE(FPU = DefaultHandler);
PROVIDE(USBD = DefaultHandler);
PROVIDE(UARTE1 = DefaultHandler);
PROVIDE(QSPI = DefaultHandler);
PROVIDE(CRYPTOCELL = DefaultHandler);
PROVIDE(PWM3 = DefaultHandler);
PROVIDE(SPIM3 = DefaultHandler);
