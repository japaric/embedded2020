# `dap-ll`

> CMSIS-DAP Load & Log

As a proof of concept the current implementation has a few artificial
limitations that could be removed with more effort:

- programs can only be loaded into RAM (the procedure of writing to Flash is
  device specific & sometimes even impossible to implement as open source code
  -- I'm looking at you, TI Cortex-R chips)

- linker sections (e.g. `.text`) must be 4-byte aligned and the start and at the
  end.

- the size of linker sections (once loaded) must not exceed `1 << 16` bytes

- and, probably, many other things I'm not even currently aware of
