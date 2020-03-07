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
