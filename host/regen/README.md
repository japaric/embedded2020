# `Regen` 🌧️

> (Yet another) Register API Generator

## Highlights

### Pay (in compile time) for want you use

Why compile the API for dozens of peripherals when your application will only
use half a dozen? Until the compiler becomes smart enough to figure this out by
itself `regen` will put peripherals behind opt-in Cargo feature. If you need a
peripheral enable its Cargo feature. All the instances of a peripheral (e.g.
`UART1`, `UART2`, etc.) will be gated behind a single Cargo feature.

### `!dereferenceable`

`regen` sidesteps issue [rust-lang/rust#55005][deferenceable] by never creating
a reference (`&[mut]-`) to MMIO registers. It's raw pointers all the way down.

[deferenceable]: https://github.com/rust-lang/rust/issues/55005

### One IR; many data formats

`regen` features an Intermediate Representation (IR) format that's designed for
optimization (merging individual, but equivalent, registers and / or bitfields
into clusters / arrays) and code generation. The IR has no defined
serialization format. Instead third parties can write translation libraries that
parse XML, C header or PDF files and translate the register data into `regen`'s
IR. Then `regen`'s public API can be used to optimize the IR and lower it to
Rust code.

### Minimal `unsafe` API

By default, `regen` considers reading and writing to any register to be safe.
Sometimes writes can produce potentially memory unsafe side effects like
unmasking an exception / interrupt (which can break critical sections), changing
the priority of an exception / interrupt (which can also break a critical
section), starting a DMA transfer (which can overwrite not owned memory).

`regen`'s IR includes a field to make register writes `unsafe`. As most data
sources do not encode potential memory unsafety, this information must be
provide by the person generating the crate; that is they must audit the safety
of all register writes. One more reason to not blindly generate code for all the
peripherals in your device!

### Singletons are always singletons

Peripheral are represented as *owned* (not global) singletons. The `regen` API
does *not* provide a method, not even a `unsafe` one, to create *more* than one
instance of any peripheral singleton as more than one instance of a singleton
ever existing would be semantically unsound. Singleton instances are created
using the `take` method; this method returns the singleton instance only once.

### Register granularity

The registers (fields) within a peripheral (`struct`) can be moved into
abstractions. This lets you finely control which registers an abstraction has
access to. Registers are also singletons so `drop`-ing them means they'll never
be modified in software again; this can be used to seal the configuration of a
peripheral. For example, if you set the baud rate of `UART1` to 115,200 and then
drop the configuration register, the application will not be able to change
the baud rate after than point.
