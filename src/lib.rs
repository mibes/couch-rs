//! # Sofa - CouchDB for Rust
//!
//! [![Crates.io](https://img.shields.io/crates/v/sofa.svg)](https://crates.io/crates/sofa)
//! [![docs.rs](https://docs.rs/sofa/badge.svg)](https://docs.rs/sofa)
//!
//! ![sofa-logo](https://raw.githubusercontent.com/YellowInnovation/sofa/master/docs/logo-sofa.png "Logo Sofa")
//!
//! ## Documentation
//!
//! Here: [http://docs.rs/sofa](http://docs.rs/sofa)
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! sofa = "0.5.1"
//! ```
//!
//! ## Description
//!
//! This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.
//!
//! Does not support `#![no_std]`
//!
//! After trying most crates for CouchDB in Rust (`chill`, `couchdb` in particular), none of them fit our needs hence the need to create our own.
//!
//! No async I/O (yet), uses a mix of Reqwest and Serde under the hood, with a few nice abstractions out there.
//!
//! **NOT 1.0 YET, so expect changes**
//!
//! **Supports CouchDB 2.0 and up.**
//!
//! Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's possible.
//!
//! ## Why the name "Sofa"
//!
//! CouchDB has a nice name, and I wanted to reflect that.
//!
//! ## License
//!
//! Licensed under either of these:
//!
//! * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
//!    [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)
//! * MIT license ([LICENSE-MIT](LICENSE-MIT) or
//!    [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
//!
//! ## Yellow Innovation
//!
//! Yellow Innovation is the innovation laboratory of the French postal service: La Poste.
//!
//! We create innovative user experiences and journeys through services with a focus on IoT lately.
//!
//! [Yellow Innovation's website and works](http://yellowinnovation.fr/en/)

extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

/// Macros that the crate exports to facilitate most of the doc-to-json-to-string-related tasks
#[allow(unused_macros)]
#[macro_use]
mod macros {
    /// Shortcut to `mod $mod; pub use mod::*;`
    macro_rules! mod_use {
        ($module:ident) => {
            mod $module;
            pub use self::$module::*;
        };
    }

    /// Extracts a JSON Value to a defined Struct
    macro_rules! json_extr {
        ($e:expr) => {
            serde_json::from_value($e.to_owned()).unwrap()
        };
    }

    /// Automatic call to serde_json::to_string() function, with prior Document::get_data() call to get documents' inner data
    macro_rules! dtj {
        ($e:expr) => {
            js!(&$e.get_data())
        };
    }

    /// Automatic call to serde_json::to_string() function
    macro_rules! js {
        ($e:expr) => {
            serde_json::to_string(&$e).unwrap()
        };
    }

    /// String creation
    macro_rules! s {
        ($e:expr) => {
            String::from($e)
        };
    }

    /// Gets milliseconds from timespec
    macro_rules! tspec_ms {
        ($tspec:ident) => {{
            $tspec.sec * 1000 + $tspec.nsec as i64 / 1000000
        }};
    }

    /// Gets current UNIX time in milliseconds
    macro_rules! msnow {
        () => {{
            let tm = time::now().to_timespec();
            tspec_ms!(tm)
        }};
    }
}

mod_use!(client);
mod_use!(database);
mod_use!(document);
mod_use!(error);
pub mod types;
mod_use!(model);

#[allow(unused_mut, unused_variables)]
#[cfg(test)]
mod sofa_tests {
    mod a_sys {
        use *;

        #[test]
        fn a_should_check_couchdbs_status() {
            let client = Client::new("http://localhost:5984".into());
            let status = client.check_status();
            assert!(status.is_some())
        }

        #[test]
        fn b_should_create_sofa_test_db() {
            let client = Client::new("http://localhost:5984".into());
            let dbw = client.db("sofa_test");
            assert!(dbw.is_ok());
        }

        #[test]
        fn c_should_create_a_document() {
            let client = Client::new("http://localhost:5984".into());
            let dbw = client.db("sofa_test");
            assert!(dbw.is_ok());
            let db = dbw.unwrap();

            let ndoc_result = db.create(json!({
                "thing": true
            }));

            assert!(ndoc_result.is_ok());

            let mut doc = ndoc_result.unwrap();
            assert_eq!(doc["thing"], json!(true))
        }

        #[test]
        fn d_should_destroy_the_db() {
            let client = Client::new("http://localhost:5984".into());
            assert!(client.destroy_db("sofa_test"));
        }
    }

    mod b_db {
        use *;

        fn setup() -> (Client, Database, Document) {
            let client = Client::new("http://localhost:5984".into());
            let dbw = client.db("sofa_test");
            assert!(dbw.is_ok());
            let db = dbw.unwrap();

            let ndoc_result = db.create(json!({
                "thing": true
            }));

            assert!(ndoc_result.is_ok());

            let mut doc = ndoc_result.unwrap();
            assert_eq!(doc["thing"], json!(true));

            (client, db, doc)
        }

        fn teardown(client: Client) {
            assert!(client.destroy_db("sofa_test"))
        }

        #[test]
        fn a_should_update_a_document() {
            let (client, db, mut doc) = setup();

            doc["thing"] = json!(false);

            let save_result = db.save(doc);
            assert!(save_result.is_ok());
            let new_doc = save_result.unwrap();
            assert_eq!(new_doc["thing"], json!(false));

            teardown(client);
        }

        #[test]
        fn b_should_remove_a_document() {
            let (client, db, doc) = setup();
            assert!(db.remove(doc));

            teardown(client);
        }

        #[test]
        fn c_should_get_a_single_document() {
            let (client, _, _) = setup();
            assert!(true);
            teardown(client);
        }

        fn setup_create_indexes() -> (Client, Database, Document) {
            let (client, db, doc) = setup();

            let spec = types::IndexFields::new(vec![types::SortSpec::Simple(s!("thing"))]);

            let res = db.insert_index("thing-index".into(), spec);

            assert!(res.is_ok());

            (client, db, doc)
        }

        #[test]
        fn d_should_create_index_in_db() {
            let (client, db, _) = setup_create_indexes();
            assert!(true);
            teardown(client);
        }

        #[test]
        fn e_should_list_indexes_in_db() {
            let (client, db, _) = setup_create_indexes();

            let index_list = db.read_indexes();
            assert!(index_list.indexes.len() > 1);
            let ref findex = index_list.indexes[1];

            assert_eq!(findex.name.as_str(), "thing-index");
            teardown(client);
        }

        #[test]
        fn f_should_ensure_index_in_db() {
            let (client, db, _) = setup();

            let spec = types::IndexFields::new(vec![types::SortSpec::Simple(s!("thing"))]);

            let res = db.ensure_index("thing-index".into(), spec);
            assert!(res.is_ok());

            teardown(client);
        }

        #[test]
        fn f_should_find_documents_in_db() {
            let (client, db, doc) = setup_create_indexes();

            let documents_res = db.find(json!({
                "selector": {
                    "thing": true
                },
                "limit": 1,
                "sort": [{
                    "thing": "desc"
                }]
            }));

            assert!(documents_res.is_ok());
            let documents = documents_res.unwrap();
            assert_eq!(documents.rows.len(), 1);

            teardown(client);
        }
    }
}
