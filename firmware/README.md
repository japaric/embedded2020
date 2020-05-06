# `firmware`

Code that's meant to be compiled for the target and not the host.

## Highlights

### Optimized `dev` builds

When developing complex programs, unoptitmized builds are usually too large to
fit in the device memory. Here, we optimize `dev` builds for size: this
minimizes program loading times during development (smaller binaries can be
loaded faster). Optimizing for size always does way less aggressive inlining;
this produces more useful stack backtraces.

### Unoptimized build dependencies

Build dependencies, dependencies used in procedural macros and build scripts,
are build without optimizations, without debug info, incrementally and with
multiple LLVM codegen units. These dependencies won't be part of the device
program so it's not important that they are fast or small. Compiling build
dependencies with these settings significantly reduces compilation times when
building from scratch (e.g. after `cargo clean`).

### Stack overflow protection

Zero-cost stack overflow protection, as described in [this blog post], is
enabled by default. The `flip-lld` linker wrapper takes care of inverting the
memory layout of the program by invoking the linker ~twice~ as many times as
necessary.

[this blog post]: https://blog.japaric.io/stack-overflow-protection/

## Running the examples

Before you can run the examples you'll need the perform the following one-time
setup.

### One-time setup

- `rustup target add thumbv7em-none-eabi`, cross compilation support

- `cargo install --git https://github.com/japaric/flip-lld`, linker wrapper that
  adds stack overflow protection

- `cd ../host && cargo install --path semidap`, tool to run embedded
  applications as if they were native applications

### Per debug session setup

- Connect the nRF52840 MDK to your PC using a USB-C cable.

### Per example steps

Just run

``` console
$ # optional
$ export RUST_LOG=semidap=info

$ # or using the rb alias: `cargo rb led`
$ cargo r --bin hello
    Finished dev [optimized + debuginfo] target(s) in 0.01s
     Running `semidap -v 0d28 -p 0204 target/thumbv7em-none-eabihf/debug/led`
[2020-02-22T16:00:00Z INFO  semidap] target: ARM Cortex-M4 (CPUID = 0x410fc241)
[2020-02-22T16:00:00Z INFO  semidap] loaded `.text` (124 B) in 7.828748ms
[2020-02-22T16:00:00Z INFO  semidap] loaded `.bss` (4 B) in 2.14817ms
[2020-02-22T16:00:00Z INFO  semidap] loaded `.vectors` (256 B) in 10.989605ms
[2020-02-22T16:00:00Z INFO  semidap] loaded 384 bytes in 21.012181ms (18275 B/s)
[2020-02-22T16:00:00Z INFO  semidap] booting program (start to end: 42.355395ms)
```
