use cm::{DCB, DWT, NVIC};
use pac::{CLOCK, P0, POWER, RTC0, USBD};

use crate::{Interrupt0, Interrupt1};

#[no_mangle]
unsafe extern "C" fn Reset() {
    // NOTE(borrow_unchecked) interrupts disabled; this runs before user code

    // use the external 32.768 KHz crystal to drive the low frequency clock
    CLOCK::borrow_unchecked(|clock| {
        // use the external crystal (LFXO) as the low-frequency clock source
        clock.LFCLKSRC.write(|w| w.SRC(1));
        // start the low-frequency clock
        clock.TASKS_LFCLKSTART.write(|w| w.TASKS_LFCLKSTART(1));
    });

    // seal some peripherals so they cannot be used from userspace
    CLOCK::seal();
    DCB::seal();
    DWT::seal();
    NVIC::seal();
    P0::seal();
    POWER::seal();
    RTC0::seal();
    USBD::seal();

    // enable interrupts (they are still masked)
    CLOCK::borrow_unchecked(|clock| {
        // 'HFXO is stable'
        clock.INTENSET.write(|w| w.HFCLKSTARTED(1));
    });
    POWER::borrow_unchecked(|power| {
        power
            .INTENSET
            .write(|w| w.USBDETECTED(1).USBREMOVED(1).USBPWRRDY(1));
    });
    USBD::borrow_unchecked(|usbd| {
        // enable interrupts
        usbd.INTENSET.write(|w| {
            w.USBRESET(1)
                .STARTED(1)
                .ENDEPIN0(1)
                .EP0DATADONE(1)
                .ENDEPOUT0(1)
                .USBEVENT(1)
                .EP0SETUP(1)
                .EPDATA(1)
        });
    });

    // configure I/O pins
    P0::borrow_unchecked(|p0| {
        // set outputs high
        p0.OUTSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));

        // set pins as output
        p0.DIRSET.write(|w| w.PIN13(1).PIN14(1).PIN15(1));
    });

    // wait for the LFXO to become stable
    CLOCK::borrow_unchecked(|clock| {
        while clock.EVENTS_LFCLKSTARTED.read().EVENTS_LFCLKSTARTED() == 0 {
            continue;
        }
    });

    // start the RTC with a counter of 0
    RTC0::borrow_unchecked(|rtc| {
        rtc.TASKS_CLEAR.write(|w| w.TASKS_CLEAR(1));
        rtc.TASKS_START.write(|w| w.TASKS_START(1));
    });

    // enable the cycle counter and start it with an initial count of 0
    DCB::borrow_unchecked(|dcb| dcb.DEMCR.rmw(|_, w| w.TRCENA(1)));
    DWT::borrow_unchecked(|dwt| {
        dwt.CYCCNT.write(0);
        dwt.CTRL.rmw(|_, w| w.CYCCNTENA(1));
    });

    // unmask interrupts
    crate::unmask0(&[Interrupt0::POWER_CLOCK]);
    crate::unmask1(&[Interrupt1::USBD]);

    extern "Rust" {
        fn main() -> !;
    }

    main()
}

#[no_mangle]
fn __semidap_timestamp() -> u32 {
    crate::cyccnt() >> 6
}

#[repr(C)]
union Vector {
    stack_pointer: *const u32,
    handler: unsafe extern "C" fn(),
    reserved: usize,
}

extern "C" {
    static __stack_top__: u32;

    // Cortex-M exceptions
    fn NMI();
    fn HardFault();
    fn MemManage();
    fn BusFault();
    fn UsageFault();
    fn SVCall();
    fn DebugMonitor();
    fn PendSV();
    fn SysTick();

    // nRF52840 interrupts
    fn POWER_CLOCK();
    fn RADIO();
    fn UARTE0_UART0();
    fn SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0();
    fn SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1();
    fn NFCT();
    fn GPIOTE();
    fn SAADC();
    fn TIMER0();
    fn TIMER1();
    fn TIMER2();
    fn RTC0();
    fn TEMP();
    fn RNG();
    fn ECB();
    fn CCM_AAR();
    fn WDT();
    fn RTC1();
    fn QDEC();
    fn COMP_LPCOMP();
    fn SWI0_EGU0();
    fn SWI1_EGU1();
    fn SWI2_EGU2();
    fn SWI3_EGU3();
    fn SWI4_EGU4();
    fn SWI5_EGU5();
    fn TIMER3();
    fn TIMER4();
    fn PWM0();
    fn PDM();
    fn MWU();
    fn PWM1();
    fn PWM2();
    fn SPIM2_SPIS2_SPI2();
    fn RTC2();
    fn I2S();
    fn FPU();
    fn USBD();
    fn UARTE1();
    fn QSPI();
    fn CRYPTOCELL();
    fn PWM3();
    fn SPIM3();
}

#[link_section = ".vectors"]
#[no_mangle]
static mut VECTORS: [Vector; 64] = [
    // Cortex-M exceptions
    Vector {
        stack_pointer: unsafe { &__stack_top__ as *const u32 },
    },
    Vector { handler: Reset },
    Vector { handler: NMI },
    Vector { handler: HardFault },
    Vector { handler: MemManage },
    Vector { handler: BusFault },
    Vector {
        handler: UsageFault,
    },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: SVCall },
    Vector {
        handler: DebugMonitor,
    },
    Vector { reserved: 0 },
    Vector { handler: PendSV },
    Vector { handler: SysTick },
    // nRF52840 interrupts
    Vector {
        handler: POWER_CLOCK, // 0
    },
    Vector {
        handler: RADIO, // 1
    },
    Vector {
        handler: UARTE0_UART0, // 2
    },
    Vector {
        handler: SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0, // 3
    },
    Vector {
        handler: SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1, // 4
    },
    Vector {
        handler: NFCT, // 5
    },
    Vector {
        handler: GPIOTE, // 6
    },
    Vector {
        handler: SAADC, // 7
    },
    Vector {
        handler: TIMER0, // 8
    },
    Vector {
        handler: TIMER1, // 9
    },
    Vector {
        handler: TIMER2, // 10
    },
    Vector {
        handler: RTC0, // 11
    },
    Vector {
        handler: TEMP, // 12
    },
    Vector {
        handler: RNG, // 13
    },
    Vector {
        handler: ECB, // 14
    },
    Vector {
        handler: CCM_AAR, // 15
    },
    Vector {
        handler: WDT, // 16
    },
    Vector {
        handler: RTC1, // 17
    },
    Vector {
        handler: QDEC, // 18
    },
    Vector {
        handler: COMP_LPCOMP, // 19
    },
    Vector {
        handler: SWI0_EGU0, // 20
    },
    Vector {
        handler: SWI1_EGU1, // 21
    },
    Vector {
        handler: SWI2_EGU2, // 22
    },
    Vector {
        handler: SWI3_EGU3, // 23
    },
    Vector {
        handler: SWI4_EGU4, // 24
    },
    Vector {
        handler: SWI5_EGU5, // 25
    },
    Vector {
        handler: TIMER3, // 26
    },
    Vector {
        handler: TIMER4, // 27
    },
    Vector {
        handler: PWM0, // 28
    },
    Vector {
        handler: PDM, // 29
    },
    Vector { reserved: 0 }, // 30
    Vector { reserved: 0 }, // 31
    Vector {
        handler: MWU, // 32
    },
    Vector {
        handler: PWM1, // 33
    },
    Vector {
        handler: PWM2, // 34
    },
    Vector {
        handler: SPIM2_SPIS2_SPI2, // 35
    },
    Vector {
        handler: RTC2, // 36
    },
    Vector {
        handler: I2S, // 37
    },
    Vector {
        handler: FPU, // 38
    },
    Vector {
        handler: USBD, // 39
    },
    Vector {
        handler: UARTE1, // 40
    },
    Vector {
        handler: QSPI, // 41
    },
    Vector {
        handler: CRYPTOCELL, // 42
    },
    Vector { reserved: 0 }, // 43
    Vector { reserved: 0 }, // 44
    Vector {
        handler: PWM3, // 45
    },
    Vector { reserved: 0 }, // 46
    Vector {
        handler: SPIM3, // 47
    },
];
