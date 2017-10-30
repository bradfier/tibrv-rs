# tibrv-rs
[![Build Status](https://travis-ci.org/bradfier/tibrv-rs.svg?branch=travis)](https://travis-ci.org/bradfier/tibrv-rs) [![Latest Version](https://img.shields.io/crates/v/tibrv.svg)](https://crates.io/crates/tibrv)

Rust bindings for TIBCO Rendezvous, a message-oriented middleware.

[Documentation](https://bradfier.github.io/tibrv-rs/)

tibrv-rs is a set of Rust bindings to the [C Implementation](https://docs.tibco.com/pub/rendezvous/8.4.0-february-2012/doc/pdf/tib_rv_c_reference.pdf)
provided by TIBCO.

tibrv-rs is still developing, and until version 1.0.0 the API is likely to change.

## License
`tibrv-rs` is licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

`TIBCO Rendezvous`, and all related components therein are property of
TIBCO Software, and are not provided with this software package.
Refer to your own TIBCO License terms for details.

## Naming
For the sake of clarity, the name of the *project* is "tibrv-rs", and the name of the *crate* is "tibrv".

The "-rs" suffix is intended to clearly separate this project from "tibrv", which is the short
name used by TIBCO for Rendezvous itself. The crate name lacks the suffix, as including it
would go against the conventions used by other projects in the Rust ecosystem.
