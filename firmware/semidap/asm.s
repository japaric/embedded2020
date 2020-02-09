  /* fn DefaultHandler() -> ! */
  .global DefaultHandler
  .cfi_sections .debug_frame
  .section .text.DefaultHandler, "ax"
  /*.type __exit, %function*/
  .thumb_func
  .cfi_startproc
  /*.align 2*/
DefaultHandler:
  bkpt 0xff
  .cfi_endproc
  .size DefaultHandler, . - DefaultHandler

  /* fn __exit(r0: i32) -> ! */
  .global __exit
  .cfi_sections .debug_frame
  .section .text.__exit, "ax"
  /*.type __exit, %function */
  .thumb_func
  .cfi_startproc
  /*.align 2*/
__exit:
  bkpt 0xab
  .cfi_endproc
  .size __exit, . - __exit

  /* fn __abort() -> ! */
  .global __abort
  .cfi_sections .debug_frame
  .section .text.__abort, "ax"
  /*.type __abort, %function*/
  .thumb_func
  .cfi_startproc
  /*.align 2 */
__abort:
  bkpt 0xaa
  .cfi_endproc
  .size __abort, . - __abort
