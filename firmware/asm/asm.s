  .global __cpsiei
  .cfi_sections .debug_frame
  .section .text.__cpsiei, "ax"
  .thumb_func
  .cfi_startproc
__cpsiei:
  cpsie i
  bx lr
  .cfi_endproc
  .size __cpsiei, . - __cpsiei

  .global __cpsidi
  .cfi_sections .debug_frame
  .section .text.__cpsidi, "ax"
  .thumb_func
  .cfi_startproc
__cpsidi:
  cpsid i
  bx lr
  .cfi_endproc
  .size __cpsidi, . - __cpsidi

  .global __sev
  .cfi_sections .debug_frame
  .section .text.__sev, "ax"
  .thumb_func
  .cfi_startproc
__sev:
  sev
  bx lr
  .cfi_endproc
  .size __sev, . - __sev

  .global __wfe
  .cfi_sections .debug_frame
  .section .text.__wfe, "ax"
  .thumb_func
  .cfi_startproc
__wfe:
  wfe
  bx lr
  .cfi_endproc
  .size __wfe, . - __wfe

  .global __wfi
  .cfi_sections .debug_frame
  .section .text.__wfi, "ax"
  .thumb_func
  .cfi_startproc
__wfi:
  wfi
  bx lr
  .cfi_endproc
  .size __wfi, . - __wfi
