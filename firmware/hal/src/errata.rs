// Reference: nRF52840_Rev_1_Errata_v1.4.pdf

#![allow(dead_code)]

/// USBD might not power up
pub unsafe fn e171a() {
    if (0x4006_EC00 as *const u32).read_volatile() == 0 {
        (0x4006_EC00 as *mut u32).write_volatile(0x9375);
    }
    (0x4006_EC14 as *mut u32).write_volatile(0xC0);
    (0x4006_EC00 as *mut u32).write_volatile(0x9375);
}

/// USBD might not power up
pub unsafe fn e171b() {
    if (0x4006_EC00 as *const u32).read_volatile() == 0 {
        (0x4006_EC00 as *mut u32).write_volatile(0x9375);
    }
    (0x4006_EC14 as *mut u32).write_volatile(0);
    (0x4006_EC00 as *mut u32).write_volatile(0x9375);
}

/// USBD cannot be enabled
pub unsafe fn e187a() {
    (0x4006_EC00 as *mut u32).write_volatile(0x9375);
    (0x4006_ED14 as *mut u32).write_volatile(3);
    (0x4006_EC00 as *mut u32).write_volatile(0x9375);
}

/// USBD cannot be enabled
pub unsafe fn e187b() {
    (0x4006_EC00 as *mut u32).write_volatile(0x9375);
    (0x4006_ED14 as *mut u32).write_volatile(0);
    (0x4006_EC00 as *mut u32).write_volatile(0x9375);
}
