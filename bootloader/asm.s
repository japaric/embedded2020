  .global __jump
  .cfi_sections .debug_frame
  .section .text.__jump, "ax"
  .thumb_func
  .syntax unified
  .cfi_startproc
__jump:
  mov lr, #0xffffffff
  msr MSP, r0
  bx  r1
  .cfi_endproc
  .size __jump, . - __jump
