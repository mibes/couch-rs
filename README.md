# CouchDB library for Rust

[![Crates.io](https://img.shields.io/crates/v/couch_rs.svg)](https://crates.io/crates/couch_rs)
[![docs.rs](https://docs.rs/couch_rs/badge.svg)](https://docs.rs/couch_rs)
![Build](https://img.shields.io/github/workflow/status/mibes/couch-rs/Rust)
![License](https://img.shields.io/crates/l/couch_rs.svg)
[![dependency status](https://deps.rs/crate/couch_rs/0.9.1/status.svg)](https://deps.rs/crate/couch_rs)
![Downloads](https://img.shields.io/crates/d/couch_rs.svg)

## Documentation

Here: [http://docs.rs/couch_rs](http://docs.rs/couch_rs)

## Installation

Include this dependency in the Cargo.toml file:

```toml
[dependencies]
couch_rs = "0.9"
```

## Description

This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.

This library is a spin-off based on the excellent work done by Mathieu Amiot and others at Yellow Innovation on the Sofa
library. The original project can be found at <https://github.com/YellowInnovation/sofa>

The Sofa library lacked support for async I/O, and missed a few essential operations we needed in our projects. That's
why I've decided to create a new project based on the original Sofa code.

The rust-rs library has been updated to the Rust 2018 edition standards, uses async I/O, and compiles against the latest
serde and reqwest libraries.

**NOT 1.0 YET, so expect changes**

**Supports CouchDB 2.3.0 and up. Used in production with various CouchDB versions, including 3.2.2.**

Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's
possible.

## Usage

A typical find operation looks like this:

```rust
use couch_rs::types::find::FindQuery;
use std::error::Error;
use serde_json::Value;
use couch_rs::document::DocumentCollection;

const DB_HOST: &str = "http://localhost:5984";
const TEST_DB: &str = "test_db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let client = couch_rs::Client::new(DB_HOST, "admin", "password")?;
  let db = client.db(TEST_DB).await?;
  let find_all = FindQuery::find_all();
  let docs = db.find_raw(&find_all).await?;
  Ok(())
}
```

## Examples

You can launch the included example with:

```shell script
cargo run --example basic_operations
```

## Running tests

Make sure that you have an instance of CouchDB 2.0+ running, either via the supplied `docker-compose.yml` file or by
yourself. It must be listening on the default port. Since Couch 3.0 the "Admin Party" mode is no longer supported. This
means you need to provide a username and password during launch. The tests and examples assume an "admin" CouchDB user
with a "password" CouchDB password. Docker run command:

```shell script
docker run --rm -p 5984:5984 -e COUCHDB_USER=admin -e COUCHDB_PASSWORD=password couchdb:3
```

And then
`cargo test --features=integration-tests -- --test-threads=1`

Single-threading the tests is very important because we need to make sure that the basic features are working before
actually testing features on dbs/documents.

If bash is available on your environment, you can also use the `test.sh` script which basically does the same thing
described above.

## License

Licensed under either of these:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

## DISCLAIMER

Please note: all content in this repository is released for use "AS IS" without any warranties of any kind, including,
but not limited to their installation, use, or performance. We disclaim any and all warranties, either express or
implied, including but not limited to any warranty of noninfringement, merchantability, and/ or fitness for a particular
purpose. We do not warrant that the technology will meet your requirements, that the operation thereof will be
uninterrupted or error-free, or that any errors will be corrected.

Any use of this library is at your own risk. There is no guarantee that it has been through thorough testing in a
comparable environment and we are not responsible for any damage or data loss incurred with their use.

You are responsible for reviewing and testing any code you run thoroughly before use in any non-testing environment.
