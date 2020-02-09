# `semidap` 🦗

> A cargo runner that makes embedded development feel like native

## Highlights

### A native feel

The `semidap` tool is a [Cargo runner][runner] that lets you run embedded
applications on real hardware as easily as you would run a native application.
No extra terminal for a separate process (e.g. `openocd`); no separate Cargo
subcommand with yet another CLI to learn; you just need the good old `cargo run`
command you already know and love.

[runner]: https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner

``` rust
fn main() -> ! {
    // This operation does NOT halt the device
    semidap::println!("Hello, world!");

    // This halts the device and terminates the `semidap` instance running
    // on the host
    semidap::exit(0);
}
```

``` console
$ cargo r --bin hello
    Finished dev [optimized + debuginfo] target(s) in 0.01s
     Running `semidap -v 0d28 -p 0204 target/$T/debug/hello`
Hello, world!
```

`semidap` is *not* an emulator. Your embedded application will run on your
development board and `semidap` will report everything (see the logging macros
in the [`semidap`](/firmware/semidap) library) to the host terminal, including
the exit code.

``` rust
fn main() -> ! {
    semidap::info!("Start"); // logs are timestamped

    semidap::debug!("working.."); // <- not included in `release`
    // pretend we are doing some work
    while time::uptime() < Duration::from_millis(1) {
        continue;
    }

    semidap::error!("Something went wrong. Exiting..");

    semidap::exit(1);
}
```

``` console
$ cargo run --bin log
  0.000000 INFO  Start
  0.000000 DEBUG working..
  0.001007 ERROR Something went wrong. Exiting..

$ echo $?
1
```

### no Flash; just RAM

`semidap` is a *development* tool; not a deployment tool. As such it doesn't
bother with non-volatile memory. It's faster to just load the program to RAM.
Short edit-compile-test turnaround times are important for development so
faster is better. `cargo watch -x run`, anyone?

``` console
$ export RUST_LOG=semidap=info

$ cargo watch -x 'run --bin led'
[Running 'cargo run --bin led']
    Finished dev [optimized + debuginfo] target(s) in 0.01s
     Running `semidap -v 0d28 -p 0204 target/$T/debug/led`
[2020-02-22T16:00:00Z INFO  semidap] target: ARM Cortex-M4 (CPUID = 0x410fc241)
[2020-02-22T16:00:00Z INFO  semidap] loaded `.text` (124 B) in 7.820227ms
[2020-02-22T16:00:00Z INFO  semidap] loaded `.bss` (4 B) in 2.14398ms
[2020-02-22T16:00:00Z INFO  semidap] loaded `.vectors` (256 B) in 10.867902ms
[2020-02-22T16:00:00Z INFO  semidap] loaded 384 bytes in 20.937522ms (18340 B/s)
[2020-02-22T16:00:00Z INFO  semidap] booting program (start to end: 42.817649ms)
[Finished running. Exit status: 0]
```

### Post-mortem debugging

Unhandled interrupt? Unaligned memory load? Stack overflow? Your program
panicked? Chances are you want to inspect the state of your program exactly at
the point of failure. `semidap` will halt the program on these scenarios, give
you all the relevant information (\*) and finally drop you to a shell for
further inspection (\*\*).

#### Unhandled exception

``` rust
fn main() -> ! {
    // this tries to read non-existent memory and causes a
    // `HardFault` (hardware) exception
    unsafe {
        (0xffff_fff0 as *const u32).read_volatile();
    }

    semidap::exit(0);
}
```

``` console
$ cargo run --bin hard-fault
     Running `semidap -v 0d28 -p 0204 target/$T/debug/hard-fault`
------------------------------------------
           unhandled exception
                HardFault

     R0: 0xfffffff0         R1: 0x0000e000
     R2: 0x00000000         R3: 0x00000000
     R4: 0x00000001         R5: 0x0000003f
     R6: 0x0000003f         R7: 0x2003fdf0
     R8: 0x00000000         R9: 0x00000000
    R10: 0x00000000        R11: 0x00000000
    R12: 0x2003fe00         SP: 0x2003fdf0
     PC: 0x2003fe08         LR: 0x2003fe57
   XPSR: 0x01000000
CONTROL: 0x00        FAULTMASK: 0x00
BASEPRI: 0x00          PRIMASK: 0x00
------------------------------------------
stack backtrace:
   0: 0x2003fe08 - main
   1: 0x2003fe56 - Reset
------------------------------------------

> help
commands:
  help                        Displays this text
  show <address> <i16>        Displays memory
  show <address> -<u16> <u16> Displays memory
  exit                        Exits the debugger
  quit                        Alias for `exit`

> show 0x2003fdf0 -4
0x2003fde0: 0x2003fe00 0x2003fe57 0x2003fe08 0x01000000
0x2003fdf0: 0x2003fdf8
```

#### Abort

The `semidap` library provides an `abort` function that terminates the `semidap`
host process with a non-zero exit code and prints the device's stack trace to
the console. The logic to walk up the stack is done on the host; this keeps the
device programs small.

``` rust
fn main() -> ! {
    foo(true);

    semidap::exit(0)
}

fn foo(recurse: bool) {
    let mut x = [0];
    let y = x.as_mut_ptr(); // use the stack
    unsafe { drop((&y as *const *mut i32).read_volatile()) }

    if recurse {
        foo(false)
    } else {
        bar()
    }
}

fn bar() {
    semidap::abort()
}
```

``` console
$ cargo run --bin abort
stack backtrace:
   0: 0x2003fe98 - __abort
   1: 0x2003fe38 - abort::bar
   2: 0x2003fe2e - abort::foo
   3: 0x2003fe28 - abort::foo
   4: 0x2003fe0a - main
   5: 0x2003fe7e - Reset

$ echo $?
134
```

The virtual unwinding done on the host can handle exceptions. `<exception
entry>` will be printed whenever the host unwinds an exception frame.

``` rust
fn main() -> ! {
    // pend `PendSV`
    SCB::borrow_unchecked(|scb| scb.ICSR.rmw(|_r, w| w.PENDSVSET(1)));

    use_the_stack();

    semidap::exit(0)
}

#[no_mangle]
fn PendSV() {
    use_the_stack();

    foo();
}

fn foo() {
    // pend `NMI`
    SCB::borrow_unchecked(|scb| scb.ICSR.rmw(|_r, w| w.NMIPENDSET(1)));

    use_the_stack();
}

#[no_mangle]
fn NMI() {
    panic!() // calls `semidap::abort`
}
```

``` console
$ cargo run --bin nested
stack backtrace:
   0: 0x2003fefa - __abort
   1: 0x2003fef4 - rust_begin_unwind
   2: 0x2003fe78 - core::panicking::panic_fmt
   3: 0x2003fe6e - core::panicking::panic
   4: 0x2003fe64 - NMI
      <exception entry>
   5: 0x2003fe4a - nested::foo
   6: 0x2003fe36 - PendSV
      <exception entry>
   7: 0x2003fe14 - main
   8: 0x2003fed4 - Reset
```

#### Stack overflow

Device programs use a reverse memory layout that prevents stack overflows from
corrupting other memory (`static` variables, constant data or program
instructions). Stack overflows will be caught and reported to the console.

``` rust
fn main() -> ! {
    fib(15);
    semidap::exit(0);
}

fn fib(n: u32) -> u32 {
    let mut x = [n; 8 * 1024]; // allocate a 32 KB buffer on the stack
    println!("SP = {:?}", x.as_mut_ptr());

    if n < 2 {
        1
    } else {
        fib(n - 1).wrapping_add(fib(n - 2))
    }
}
```

``` console
$ cargo run --bin stack-overflow
SP = 0x200379cc
SP = 0x2002f9a4
SP = 0x2002797c
SP = 0x2001f954
SP = 0x2001792c
SP = 0x2000f904
SP = 0x200078dc

------------------------------------------
         stack overflow detected

     R0: 0x00000000         R1: 0x1ffff8b4
     R2: 0x00000000         R3: 0x200078d0
     R4: 0x00000008         R5: 0x2000f8de
     R6: 0x00000032         R7: 0x200078d0
     R8: 0x00000000         R9: 0x00000000
    R10: 0x00000000        R11: 0x00000000
    R12: 0x2003fa00         SP: 0x1ffff8b0
CONTROL: 0x00        FAULTMASK: 0x00
BASEPRI: 0x00          PRIMASK: 0x00
------------------------------------------

> show 0x2000f904 -11
0x2000f8d0:                       0x00000009 0x78300000
0x2000f8e0: 0x30303032 0x63643837 0x00000000 0x0000000a
0x2000f8f0: 0x20017906 0x00000032 0x20017920 0x2003fad1
0x2000f900: 0x0a0a0a0a 0x0000000a
```

No stack backtrace is shown when stack overflows are caught because by the time
the protection mechanism kicks in the information required to unwind the stack
(Link Register and Program Counter) has already been lost.

#### Panics

The default panicking behavior is to call `semidap::abort`. The panic message is
discarded.

``` rust
fn main() -> ! {
    panic!()
}

stack backtrace:
   0: 0x2003fe84 - __abort
   1: 0x2003fe80 - rust_begin_unwind
   2: 0x2003fe1c - core::panicking::panic_fmt
   3: 0x2003fe12 - core::panicking::panic
   4: 0x2003fe08 - main
   5: 0x2003fe62 - Reset
```
