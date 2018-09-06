# Sofa - CouchDB for Rust

[![Crates.io](https://img.shields.io/crates/v/sofa.svg)](https://crates.io/crates/sofa)[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa?ref=badge_shield)

[![docs.rs](https://docs.rs/sofa/badge.svg)](https://docs.rs/sofa)

![sofa-logo](https://raw.githubusercontent.com/YellowInnovation/sofa/master/docs/logo-sofa.png "Logo Sofa")

## Documentation

Here: [http://docs.rs/sofa](http://docs.rs/sofa)

## Installation

```toml
[dependencies]
sofa = "0.6"
```

## Description

This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.

Does not support `#![no_std]`

After trying most crates for CouchDB in Rust (`chill`, `couchdb` in particular), none of them fit our needs hence the need to create our own.

No async I/O (yet), uses a mix of Reqwest and Serde under the hood, with a few nice abstractions out there.

**NOT 1.0 YET, so expect changes**

**Supports CouchDB 2.0 and up.**

Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's possible.

## Running tests

Make sure that you have an instance of CouchDB 2.0+ running, either via the supplied `docker-compose.yml` file or by yourself. It must be listening on the default port.

And then
`cargo test -- --test-threads=1`

Single-threading the tests is very important because we need to make sure that the basic features are working before actually testing features on dbs/documents.

## Why the name "Sofa"

CouchDB has a nice name, and I wanted to reflect that.

## License

Licensed under either of these:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
   [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))


[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa?ref=badge_large)

## Yellow Innovation

Yellow Innovation is the innovation laboratory of the French postal service: La Poste.

We create innovative user experiences and journeys through services with a focus on IoT lately.

[Yellow Innovation's website and works](http://yellowinnovation.fr/en/)