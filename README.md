Hey, how time flies! It's now 2021 (at time of writing). This experiment is over and the repository has been archived. Some of the ideas here live on as part of the [knurling project](https://github.com/knurling-rs).

---

# `embedded2020`

> A fresh look at embedded Rust development

## Please read this

The goal of this personal experiment is exploring ways in which developing
embedded Rust software (HALs, device-agnostic libraries, applications, etc.) can
be improved. You'll see me here re-implementing things that have already been
published on crates.io (specially things that require reading a manual or a
standard). That's pretty much intentional. The goal is improving how those
things are developed so pulling them as dependencies from crates.io would defeat
the purpose. IOW, the goal here is studying the *process* of making software and
not just making some application or library.

All the code in this repository is a proof of concept. I have no intentions of
making it more general (i.e. supporting more devices, probes, architectures,
etc.) than what's necessary for my experiments. That is to say: depend on things
in this repository at your own risk and do not expect any support from me.

Also, the code in this repository is pretty opinionated; feel free to disagree
with any or all of it.

Finally, what's written in the READMEs may already be implemented or it may just
be planned. I'm not going to bother to regularly update the status of things in
the READMEs.

## Highlights

### Code organization

- The `firmware` folder is a workspace that contains `no_std` crates that will
  be compiled for the target. Running any Cargo command within that folder
  automatically does cross compilation (see `firmware/.cargo/config`).

- The `host` folder is a workspace that contains `std` crates that are meant to
  be compiled for and executed on the host. These crates can not be cross
  compiled to the target (because they depend on `std`).

- The `shared` folder contains `no_std` crates that either (a) are meant to be
  compiled for the target but have been put in this folder so they can be easily
  unit tested on the host (`cargo t`) or (b) serve to share data (e.g.
  constants) between the firmware and code that will run on the host.

### Cargo aliases

- `.cargo/config` defines a bunch of aliases. `cargo run --release --example
  foo` is too long so instead you can type `cargo rre foo`. These aliases are
  available in all workspaces.

## License

All source code (including code snippets) is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0][L1])
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/licenses/MIT][L2])

[L1]: https://www.apache.org/licenses/LICENSE-2.0
[L2]: https://opensource.org/licenses/MIT

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
