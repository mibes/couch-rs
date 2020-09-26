//! # Sofa - CouchDB for Rust
//!
//! [![Crates.io](https://img.shields.io/crates/v/sofa.svg)](https://crates.io/crates/sofa)
//! [![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa?ref=badge_shield)
//!
//! [![docs.rs](https://docs.rs/sofa/badge.svg)](https://docs.rs/sofa)
//!
//! ![sofa-logo](https://raw.githubusercontent.com/mibes/sofa/master/docs/logo-sofa.png "Logo Sofa")
//!
//! ## Documentation
//!
//! Here: [http://docs.rs/sofa](http://docs.rs/sofa)
//!
//! ## Installation
//!
//! If you want to use this particular fork, include this dependency in the Cargo.toml file:
//! ```toml
//! [dependencies.sofa]
//! git = "https://github.com/mibes/sofa.git"
//! ```
//!
//! If you want to continue to use the "old" 0.6 version use this dependency instead:
//! ```toml
//! [dependencies]
//! sofa = "0.6"
//! ```
//!
//! ## Description
//!
//! This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.
//!
//! After trying most crates for CouchDB in Rust (`chill`, `couchdb` in particular), none of them fit our needs hence the need to create our own.
//!
//! Uses async I/O, with a mix of Reqwest and Serde under the hood, and a few nice abstractions out there.
//!
//! **NOT 1.0 YET, so expect changes**
//!
//! **Supports CouchDB 2.3.0 and up.**
//!
//! Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's possible.
//!
//! The 0.7 version is based on the 0.6 release from https://github.com/YellowInnovation/sofa.
//! It has been updated to the Rust 2018 edition standards, uses async I/O, and compiles against the latest serde and
//! reqwest libraries.
//!
//! ## Example code
//!
//! You can launch the included example with:
//! ```shell script
//! cargo run --example basic_operations
//! ```
//!
//! ## Running tests
//!
//! Make sure that you have an instance of CouchDB 2.0+ running, either via the supplied `docker-compose.yml` file or by yourself. It must be listening on the default port.
//!
//! And then
//! `cargo test -- --test-threads=1`
//!
//! Single-threading the tests is very important because we need to make sure that the basic features are working before actually testing features on dbs/documents.
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
//!
//! [![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2FYellowInnovation%2Fsofa?ref=badge_large)
//!
//! ## Yellow Innovation
//!
//! Yellow Innovation is the innovation laboratory of the French postal service: La Poste.
//!
//! We create innovative user experiences and journeys through services with a focus on IoT lately.
//!
//! [Yellow Innovation's website and works](http://yellowinnovation.fr/en/)

/// Macros that the crate exports to facilitate most of the
/// doc-to-json-to-string-related tasks
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

    /// Extracts a JSON Value to a defined Struct; Returns the default value when the field can not be found
    /// or converted
    macro_rules! json_extr {
        ($e:expr) => {
            serde_json::from_value($e.to_owned()).unwrap_or_default()
        };
    }

    /// Automatic call to serde_json::to_string() function, with prior
    /// Document::get_data() call to get documents' inner data
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

mod client;
pub mod database;
pub mod document;
pub mod error;
pub mod model;
pub mod types;

pub use client::Client;

#[allow(unused_mut, unused_variables)]
#[cfg(test)]
mod sofa_tests {
    mod a_sys {
        const DB_HOST: &str = "http://admin:password@localhost:5984";

        use crate::client::Client;
        use serde_json::json;

        #[tokio::test]
        async fn a_should_check_couchdbs_status() {
            let client = Client::new(DB_HOST).unwrap();
            let status = client.check_status().await;
            assert!(status.is_ok());
        }

        #[tokio::test]
        async fn b_should_create_sofa_test_db() {
            let client = Client::new(DB_HOST).unwrap();
            let dbw = client.db("b_should_create_sofa_test_db").await;
            assert!(dbw.is_ok());

            let _ = client.destroy_db("b_should_create_sofa_test_db");
        }

        #[tokio::test]
        async fn c_should_create_a_document() {
            let client = Client::new(DB_HOST).unwrap();
            let dbw = client.db("c_should_create_a_document").await;
            assert!(dbw.is_ok());
            let db = dbw.unwrap();

            let ndoc_result = db
                .create(json!({
                    "thing": true
                }))
                .await;

            assert!(ndoc_result.is_ok());

            let mut doc = ndoc_result.unwrap();
            assert_eq!(doc["thing"], json!(true));

            let _ = client.destroy_db("c_should_create_a_document");
        }

        #[tokio::test]
        async fn d_should_destroy_the_db() {
            let client = Client::new(DB_HOST).unwrap();
            let _ = client.db("d_should_destroy_the_db").await;

            assert!(client.destroy_db("d_should_destroy_the_db").await.unwrap());
        }
    }

    mod b_db {
        use crate::client::Client;
        use crate::database::Database;
        use crate::document::Document;
        use crate::types;
        use crate::types::find::FindQuery;
        use crate::types::query::QueryParams;
        use serde_json::json;

        const DB_HOST: &str = "http://admin:password@localhost:5984";

        async fn setup(dbname: &str) -> (Client, Database, Document) {
            let client = Client::new(DB_HOST).unwrap();
            let dbw = client.db(dbname).await;
            assert!(dbw.is_ok());
            let db = dbw.unwrap();

            let ndoc_result = db
                .create(json!({
                    "thing": true
                }))
                .await;

            assert!(ndoc_result.is_ok());

            let mut doc = ndoc_result.unwrap();
            assert_eq!(doc["thing"], json!(true));

            (client, db, doc)
        }

        async fn teardown(client: Client, dbname: &str) {
            assert!(client.destroy_db(dbname).await.unwrap())
        }

        #[tokio::test]
        async fn a_should_update_a_document() {
            let (client, db, mut doc) = setup("a_should_update_a_document").await;

            doc["thing"] = json!(false);

            let save_result = db.save(doc).await;
            assert!(save_result.is_ok());
            let new_doc = save_result.unwrap();
            assert_eq!(new_doc["thing"], json!(false));

            teardown(client, "a_should_update_a_document").await;
        }

        #[tokio::test]
        async fn b_should_remove_a_document() {
            let (client, db, doc) = setup("b_should_remove_a_document").await;
            assert!(db.remove(doc).await);

            teardown(client, "b_should_remove_a_document").await;
        }

        #[tokio::test]
        async fn c_should_get_a_single_document() {
            let (client, ..) = setup("c_should_get_a_single_document").await;
            teardown(client, "c_should_get_a_single_document").await;
        }

        async fn setup_create_indexes(dbname: &str) -> (Client, Database, Document) {
            let (client, db, doc) = setup(dbname).await;

            let spec = types::index::IndexFields::new(vec![types::find::SortSpec::Simple(s!("thing"))]);

            let res = db.insert_index("thing-index".into(), spec).await;

            assert!(res.is_ok());

            (client, db, doc)
        }

        #[tokio::test]
        async fn d_should_create_index_in_db() {
            let (client, db, _) = setup_create_indexes("d_should_create_index_in_db").await;
            teardown(client, "d_should_create_index_in_db").await;
        }

        #[tokio::test]
        async fn e_should_list_indexes_in_db() {
            let (client, db, _) = setup_create_indexes("e_should_list_indexes_in_db").await;

            let index_list = db.read_indexes().await.unwrap();
            assert!(index_list.indexes.len() > 1);
            let findex = &index_list.indexes[1];

            assert_eq!(findex.name.as_str(), "thing-index");
            teardown(client, "e_should_list_indexes_in_db").await;
        }

        #[tokio::test]
        async fn f_should_ensure_index_in_db() {
            let (client, db, _) = setup("f_should_ensure_index_in_db").await;

            let spec = types::index::IndexFields::new(vec![types::find::SortSpec::Simple(s!("thing"))]);

            let res = db.ensure_index("thing-index".into(), spec).await;
            assert!(res.is_ok());

            teardown(client, "f_should_ensure_index_in_db").await;
        }

        #[tokio::test]
        async fn g_should_find_documents_in_db() {
            let (client, db, doc) = setup_create_indexes("g_should_find_documents_in_db").await;
            let query = FindQuery::new_from_value(json!({
                "selector": {
                    "thing": true
                },
                "limit": 1,
                "sort": [{
                    "thing": "desc"
                }]
            }));

            let documents_res = db.find(&query).await;

            assert!(documents_res.is_ok());
            let documents = documents_res.unwrap();
            assert_eq!(documents.rows.len(), 1);

            teardown(client, "g_should_find_documents_in_db").await;
        }

        #[tokio::test]
        async fn h_should_bulk_get_a_document() {
            let (client, db, doc) = setup("h_should_bulk_get_a_document").await;
            let id = doc._id.clone();

            let collection = db.get_bulk(vec![id]).await.unwrap();
            assert_eq!(collection.rows.len(), 1);
            assert!(db.remove(doc).await);

            teardown(client, "h_should_bulk_get_a_document").await;
        }

        #[tokio::test]
        async fn i_should_bulk_get_invalid_documents() {
            let (client, db, doc) = setup("i_should_bulk_get_invalid_documents").await;
            let id = doc._id.clone();
            let invalid_id = "does_not_exist".to_string();

            let collection = db.get_bulk(vec![id, invalid_id]).await.unwrap();
            assert_eq!(collection.rows.len(), 1);
            assert!(db.remove(doc).await);

            teardown(client, "i_should_bulk_get_invalid_documents").await;
        }

        #[tokio::test]
        async fn j_should_get_all_documents_with_keys() {
            let (client, db, doc) = setup("j_should_get_all_documents_with_keys").await;
            let id = doc._id.clone();

            let params = QueryParams::from_keys(vec![id]);

            let collection = db.get_all_params(Some(params)).await.unwrap();
            assert_eq!(collection.rows.len(), 1);
            assert!(db.remove(doc).await);

            teardown(client, "j_should_get_all_documents_with_keys").await;
        }
    }
}
