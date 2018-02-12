//! # Sofa - CouchDB for Rust
//!
//! This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.
//!
//! Does not support `#![no_std]`
//!
//! Supports CouchDB 2.0 and up.
//!
//! Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's possible.
extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

#[cfg(test)]
#[macro_use] extern crate pretty_assertions;

/// Macros that the crate exports to facilitate most of the doc-to-json-to-string-related tasks
#[allow(unused_macros)]
#[macro_use] mod macros {
    /// Extracts a JSON Value to a defined Struct
    macro_rules! json_extr {
        ($e: expr) => (serde_json::from_value($e.to_owned()).unwrap())
    }

    /// Automatic call to serde_json::to_string() function, with prior Document::get_data() call to get documents' inner data
    macro_rules! dtj {
        ($e: expr) => (js!(&$e.get_data()))
    }

    /// Automatic call to serde_json::to_string() function
    macro_rules! js {
        ($e: expr) => (serde_json::to_string(&$e).unwrap())
    }

    /// String creation
    macro_rules! s {
        ($e: expr) => (String::from($e))
    }

    /// Gets milliseconds from timespec
    macro_rules! tspec_ms {
        ($tspec: ident) => ({
            $tspec.sec * 1000 + $tspec.nsec as i64 / 1000000
        })
    }

    /// Gets current UNIX time in milliseconds
    macro_rules! msnow {
        () => ({
            let tm = time::now().to_timespec();
            tspec_ms!(tm)
        })
    }
}

mod client;
pub use client::*;

mod database;
pub use database::*;

mod document;
pub use document::*;

mod error;
pub use error::*;

pub mod types;

mod model;
pub use model::*;

#[allow(unused_mut, unused_variables)]
#[cfg(test)]
mod sofa_tests {

    mod a_sys {
        use ::*;

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
                "pouet": true
            }));

            assert!(ndoc_result.is_ok());

            let mut doc = ndoc_result.unwrap();
            assert_eq!(doc["pouet"], json!(true))
        }

        #[test]
        fn d_should_destroy_the_db() {
            let client = Client::new("http://localhost:5984".into());
            assert!(client.destroy_db("sofa_test"));
        }
    }

    mod b_db {
        use ::*;


        fn setup() -> (Client, Database, Document) {
            let client = Client::new("http://localhost:5984".into());
            let dbw = client.db("sofa_test");
            assert!(dbw.is_ok());
            let db = dbw.unwrap();

            let ndoc_result = db.create(json!({
                "pouet": true
            }));

            assert!(ndoc_result.is_ok());

            let mut doc = ndoc_result.unwrap();
            assert_eq!(doc["pouet"], json!(true));

            (client, db, doc)
        }

        fn teardown(client: Client) {
            assert!(client.destroy_db("sofa_test"))
        }

        #[test]
        fn a_should_update_a_document() {
            let (client, db, mut doc) = setup();

            doc["pouet"] = json!(false);

            let save_result = db.save(doc);
            assert!(save_result.is_ok());
            let new_doc = save_result.unwrap();
            assert_eq!(new_doc["pouet"], json!(false));

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

            let spec = IndexFields::new(vec![
                SortSpec::Simple(s!("pouet"))
            ]);

            let res = db.insert_index("pouet-index".into(), spec);

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

            assert_eq!(findex.name.as_str(), "pouet-index");
            teardown(client);
        }

        #[test]
        fn f_should_ensure_index_in_db() {
            let (client, db, _) = setup();

            let spec = IndexFields::new(vec![
                SortSpec::Simple(s!("pouet"))
            ]);

            let res = db.ensure_index("pouet-index".into(), spec);
            assert!(res.is_ok());


            teardown(client);
        }

        #[test]
        fn f_should_find_documents_in_db() {
            let (client, db, doc) = setup_create_indexes();

            let documents_res = db.find(json!({
                "selector": {
                    "pouet": true
                },
                "limit": 1,
                "sort": [{
                    "pouet": "desc"
                }]
            }));

            assert!(documents_res.is_ok());
            let documents = documents_res.unwrap();
            assert_eq!(documents.rows.len(), 1);

            teardown(client);
        }
    }
}
