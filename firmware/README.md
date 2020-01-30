# `firmware`

Code that's meant to be compiled for the target and not the host.

## Running examples

Before you can run the examples you'll need the perform the following one-time
setup.

### One-time setup

- `rustup target add thumbv7em-none-eabihf`

- Flash the bootloader. See instructions in the `bootloader` directory.

- `cd ../host && cargo install --path dap-ll`

### Per debug session setup

- Connect the Particle debugger to the Xenon using the ribbon cable. Then plug
  the Particle debugger into one of the host's USB ports.

### Per example steps

Just run

``` console
$ cargo r --example leds --release
```
