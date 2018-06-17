# Sofa - CouchDB for Rust

[![Crates.io](https://img.shields.io/crates/v/sofa.svg)](https://crates.io/crates/sofa)

## Description
This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.

Does not support `#![no_std]`

After trying most crates for CouchDB in Rust (`chill`, `couchdb` in particular), none of them fit our needs hence the need to create our own.

No async I/O, uses a mix of Reqwest and Serde under the hood, with a few nice abstractions out there.

**NOT 1.0 YET, so expect changes**

**Supports CouchDB 2.0 and up.**

Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's possible.

## Why the name "Sofa"?
CouchDB has a nice name, and I wanted to reflect that.

## Documentation
Here: http://docs.rs/sofa

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

## Yellow Innovation
Yellow Innovation is the innovation laboratory of the French postal service: La Poste.

We create innovative user experiences and journeys through services with a focus on IoT lately.

[Yellow Innovation's website and works](http://yellowinnovation.fr/en/)
