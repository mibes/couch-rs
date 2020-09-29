//! # CouchDB library for Rust
//!
//! ## Description
//!
//! This crate is an interface to CouchDB HTTP REST API. Works with stable Rust.
//!
//! This library is a spin-off based on the excellent work done by Mathieu Amiot and others at Yellow Innovation on the
//! Sofa library. The original project can be found at https://github.com/YellowInnovation/sofa
//!
//! The Sofa library lacked support for async I/O, and missed a few essential operations we needed in our projects. That's
//! why I've decided to create a new project based on the original Sofa code.
//!
//! The rust-rs library has been updated to the Rust 2018 edition standards, uses async I/O, and compiles against the latest serde and
//! reqwest libraries.
//!
//! **NOT 1.0 YET, so expect changes**
//!
//! **Supports CouchDB 2.3.0 and up, including the newly released 3.0 version.**
//!
//! Be sure to check [CouchDB's Documentation](http://docs.couchdb.org/en/latest/index.html) in detail to see what's possible.
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
//! Since Couch 3.0 the "Admin Party" mode is no longer supported. This means you need to provide a username and password during launch.
//! The tests and examples assume an "admin" CouchDB user with a "password" CouchDB password. Docker run command:
//!
//! ```shell script
//! docker run --rm -p 5984:5984 -e COUCHDB_USER=admin -e COUCHDB_PW=password couchdb:3
//! ```
//!
//! And then
//! `cargo test -- --test-threads=1`
//!
//! Single-threading the tests is very important because we need to make sure that the basic features are working before actually testing features on dbs/documents.
//!
//! ## Usage
//!
//! A typical find operation looks like this.
//!
//! ```
//! use couch_rs::types::find::FindQuery;
//! use std::error::Error;
//!
//! const DB_HOST: &str = "http://admin:password@localhost:5984";
//! const TEST_DB: &str = "test_db";
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let client = couch_rs::Client::new_local_test()?;
//!     let db = client.db(TEST_DB).await?;
//!     let find_all = FindQuery::find_all();
//!     let docs = db.find(&find_all).await?;
//!     Ok(())
//! }
//!```
//! See the `database` module for additional usage examples. Or have a look at the `examples` in the
//! GitHub repositiory.
//!

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
/// Database operations on a CouchDB Database.
pub mod database;
/// Document model to support CouchDB document operations.
pub mod document;
/// Error wrappers for the HTTP status codes returned by CouchDB.
pub mod error;
/// Trait that provides methods that can be used to switch between abstract Document and
/// concrete Model implementors (such as your custom data models)
pub mod model;
/// Data types to support CouchDB operations.
pub mod types;

pub use client::Client;

#[allow(unused_mut, unused_variables)]
#[cfg(test)]
mod couch_rs_tests {

    mod client_tests {
        use crate::client::Client;
        use reqwest::StatusCode;
        use serde_json::json;

        #[tokio::test]
        async fn should_check_couchdbs_status() {
            let client = Client::new_local_test().unwrap();
            let status = client.check_status().await;
            assert!(status.is_ok());
        }

        #[tokio::test]
        async fn should_create_test_db() {
            let client = Client::new_local_test().unwrap();
            let dbw = client.db("should_create_test_db").await;
            assert!(dbw.is_ok());

            let _ = client.destroy_db("should_create_test_db");
        }

        #[tokio::test]
        async fn should_create_a_document() {
            let client = Client::new_local_test().unwrap();
            let dbw = client.db("should_create_a_document").await;
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

            let _ = client.destroy_db("should_create_a_document");
        }

        #[tokio::test]
        async fn should_create_bulk_documents() {
            let client = Client::new_local_test().unwrap();
            let dbname = "should_create_bulk_documents";
            let dbw = client.db(dbname).await;
            assert!(dbw.is_ok());
            let db = dbw.unwrap();

            let ndoc_result = db
                .bulk_docs(vec![
                    json!({
                        "_id":"first",
                        "thing": true
                    }),
                    json!({
                        "_id":"first",
                        "thing": false
                    }),
                ])
                .await;

            assert!(ndoc_result.is_ok());

            let mut docs = ndoc_result.unwrap();
            let mut docs = docs.into_iter();
            let first_result = docs.next().unwrap();
            assert!(first_result.is_ok());
            assert!(first_result.unwrap().rev.is_some());

            let second_result = docs.next().unwrap();
            assert!(second_result.is_err());
            assert_eq!(second_result.err().unwrap().status, StatusCode::CONFLICT);

            let _ = client.destroy_db(dbname);
        }

        #[tokio::test]
        async fn should_destroy_the_db() {
            let client = Client::new_local_test().unwrap();
            let _ = client.db("should_destroy_the_db").await;

            assert!(client.destroy_db("should_destroy_the_db").await.unwrap());
        }
    }

    mod database_tests {
        use crate::client::Client;
        use crate::database::Database;
        use crate::document::{Document, DocumentCollection};
        use crate::types;
        use crate::types::find::FindQuery;
        use crate::types::query::{QueriesParams, QueryParams};
        use crate::types::view::{CouchFunc, CouchViews};
        use serde_json::{json, Value};
        use tokio::sync::mpsc;
        use tokio::sync::mpsc::{Receiver, Sender};

        async fn setup(dbname: &str) -> (Client, Database, Document) {
            let client = Client::new_local_test().unwrap();
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

        async fn setup_multiple(dbname: &str, nr_of_docs: usize) -> (Client, Database, Vec<Document>) {
            let client = Client::new_local_test().unwrap();
            let dbw = client.db(dbname).await;
            assert!(dbw.is_ok());
            let db = dbw.unwrap();
            let mut docs = vec![];

            for _ in 0..nr_of_docs {
                let ndoc_result = db
                    .create(json!({
                        "thing": true
                    }))
                    .await;

                assert!(ndoc_result.is_ok());

                let mut doc = ndoc_result.unwrap();
                assert_eq!(doc["thing"], json!(true));
                docs.push(doc)
            }

            (client, db, docs)
        }

        async fn teardown(client: Client, dbname: &str) {
            assert!(client.destroy_db(dbname).await.unwrap())
        }

        #[tokio::test]
        async fn should_update_a_document() {
            let (client, db, mut doc) = setup("should_update_a_document").await;

            doc["thing"] = json!(false);

            let save_result = db.save(doc).await;
            assert!(save_result.is_ok());
            let new_doc = save_result.unwrap();
            assert_eq!(new_doc["thing"], json!(false));

            teardown(client, "should_update_a_document").await;
        }

        #[tokio::test]
        async fn should_handle_a_document_plus() {
            let dbname = "should_handle_a_document_plus";
            let (client, db, mut doc) = setup(dbname).await;

            assert!(db.remove(doc).await);
            // make sure db is empty
            assert_eq!(db.get_all().await.unwrap().rows.len(), 0);

            // create 1 doc with plus sign in the _id
            let id = "1+2";
            let created = db.create(json!({ "_id": id })).await.unwrap();
            assert_eq!(created._id, id);

            // update it
            let save_result = db.save(created.clone()).await;
            assert!(save_result.is_ok());
            // make sure db has only 1 doc
            assert_eq!(db.get_all().await.unwrap().rows.len(), 1);

            // delete it
            assert!(db.remove(save_result.unwrap()).await);
            // make sure db has no docs
            assert_eq!(db.get_all().await.unwrap().rows.len(), 0);

            teardown(client, dbname).await;
        }

        #[tokio::test]
        async fn should_remove_a_document() {
            let (client, db, doc) = setup("should_remove_a_document").await;
            assert!(db.remove(doc).await);

            teardown(client, "should_remove_a_document").await;
        }

        #[tokio::test]
        async fn should_get_a_single_document() {
            let (client, ..) = setup("should_get_a_single_document").await;
            teardown(client, "should_get_a_single_document").await;
        }

        async fn setup_create_indexes(dbname: &str) -> (Client, Database, Document) {
            let (client, db, doc) = setup(dbname).await;

            let spec = types::index::IndexFields::new(vec![types::find::SortSpec::Simple(s!("thing"))]);

            let res = db.insert_index("thing-index", spec).await;

            assert!(res.is_ok());

            (client, db, doc)
        }

        #[tokio::test]
        async fn should_create_index_in_db() {
            let (client, db, _) = setup_create_indexes("should_create_index_in_db").await;
            teardown(client, "should_create_index_in_db").await;
        }

        #[tokio::test]
        async fn should_list_indexes_in_db() {
            let (client, db, _) = setup_create_indexes("should_list_indexes_in_db").await;

            let index_list = db.read_indexes().await.unwrap();
            assert!(index_list.indexes.len() > 1);
            let findex = &index_list.indexes[1];

            assert_eq!(findex.name.as_str(), "thing-index");
            teardown(client, "should_list_indexes_in_db").await;
        }

        #[tokio::test]
        async fn should_ensure_index_in_db() {
            let (client, db, _) = setup("should_ensure_index_in_db").await;

            let spec = types::index::IndexFields::new(vec![types::find::SortSpec::Simple(s!("thing"))]);

            let res = db.ensure_index("thing-index", spec).await;
            assert!(res.is_ok());

            teardown(client, "should_ensure_index_in_db").await;
        }

        #[tokio::test]
        async fn should_find_documents_in_db() {
            let (client, db, doc) = setup_create_indexes("should_find_documents_in_db").await;
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

            teardown(client, "should_find_documents_in_db").await;
        }

        #[tokio::test]
        async fn should_bulk_get_a_document() {
            let (client, db, doc) = setup("should_bulk_get_a_document").await;
            let id = doc._id.clone();

            let collection = db.get_bulk(vec![id]).await.unwrap();
            assert_eq!(collection.rows.len(), 1);
            assert!(db.remove(doc).await);

            teardown(client, "should_bulk_get_a_document").await;
        }

        #[tokio::test]
        async fn should_bulk_get_invalid_documents() {
            let (client, db, doc) = setup("should_bulk_get_invalid_documents").await;
            let id = doc._id.clone();
            let invalid_id = "does_not_exist".to_string();

            let collection = db.get_bulk(vec![id, invalid_id]).await.unwrap();
            assert_eq!(collection.rows.len(), 1);
            assert!(db.remove(doc).await);

            teardown(client, "should_bulk_get_invalid_documents").await;
        }

        #[tokio::test]
        async fn should_get_all_documents_with_keys() {
            let (client, db, doc) = setup("should_get_all_documents_with_keys").await;
            let id = doc._id.clone();

            let params = QueryParams::from_keys(vec![id]);

            let collection = db.get_all_params(Some(params)).await.unwrap();
            assert_eq!(collection.rows.len(), 1);
            assert!(db.remove(doc).await);

            teardown(client, "should_get_all_documents_with_keys").await;
        }

        #[tokio::test]
        async fn should_query_documents_with_keys() {
            let db_name = "should_query_documents_with_keys";
            let (client, db, doc) = setup(db_name).await;
            let id = doc._id.clone();
            let view_name = "testViewAll";
            db.create_view(
                view_name,
                CouchViews::new(
                    view_name,
                    CouchFunc {
                        map: r#"function(doc) {{
                                    emit(doc._id, null);
                            }}"#
                        .to_string(),
                        reduce: None,
                    },
                ),
            )
            .await
            .unwrap();
            let ndoc = db
                .create(json!({
                    "thing": true
                }))
                .await
                .unwrap();
            let ndoc_id = ndoc._id.clone();
            let single_view_name = "testViewSingle";
            db.create_view(
                single_view_name,
                CouchViews::new(
                    single_view_name,
                    CouchFunc {
                        map: format!(
                            r#"function(doc) {{
                                    if(doc._id === "{}") {{
                                        emit(doc._id, null);
                                    }}
                            }}"#,
                            ndoc_id
                        )
                        .to_string(),
                        reduce: None,
                    },
                ),
            )
            .await
            .unwrap();

            // executing 'all' view querying with keys containing 1 key should result in 1 and 0 entries, respectively
            assert_eq!(
                db.query(view_name, view_name, Some(QueryParams::from_keys(vec![id.clone()])))
                    .await
                    .unwrap()
                    .rows
                    .len(),
                1
            );
            assert_eq!(
                db.query(
                    single_view_name,
                    single_view_name,
                    Some(QueryParams::from_keys(vec![id])),
                )
                .await
                .unwrap()
                .rows
                .len(),
                0
            );

            assert!(db.remove(ndoc).await);
            assert!(db.remove(doc).await);

            teardown(client, db_name).await;
        }

        #[tokio::test]
        async fn should_query_documents_with_key() {
            let db_name = "should_query_documents_with_key";
            let (client, db, doc) = setup(db_name).await;
            let id = doc._id.clone();
            let view_name = "testViewAll";
            db.create_view(
                view_name,
                CouchViews::new(
                    view_name,
                    CouchFunc {
                        map: r#"function(doc) {{
                                    emit(doc._id, null);
                            }}"#
                        .to_string(),
                        reduce: None,
                    },
                ),
            )
            .await
            .unwrap();
            let ndoc = db
                .create(json!({
                    "thing": true
                }))
                .await
                .unwrap();
            let ndoc_id = ndoc._id.clone();
            let single_view_name = "testViewSingle";
            db.create_view(
                single_view_name,
                CouchViews::new(
                    single_view_name,
                    CouchFunc {
                        map: format!(
                            r#"function(doc) {{
                                    if(doc._id === "{}") {{
                                        emit(doc._id, null);
                                    }}
                            }}"#,
                            ndoc_id
                        )
                        .to_string(),
                        reduce: None,
                    },
                ),
            )
            .await
            .unwrap();

            // executing 'all' view querying with a specific key should result in 1 and 0 entries, respectively
            let mut one_key = QueryParams::default();
            one_key.key = Some(doc._id.clone());

            assert_eq!(
                db.query(view_name, view_name, Some(one_key.clone()))
                    .await
                    .unwrap()
                    .rows
                    .len(),
                1
            );
            assert_eq!(
                db.query(single_view_name, single_view_name, Some(one_key))
                    .await
                    .unwrap()
                    .rows
                    .len(),
                0
            );

            assert!(db.remove(ndoc).await);
            assert!(db.remove(doc).await);

            teardown(client, db_name).await;
        }

        #[tokio::test]
        async fn should_query_documents_with_defaultparams() {
            let dbname = "should_query_documents_with_defaultparams";
            let (client, db, doc) = setup(dbname).await;
            let id = doc._id.clone();
            let view_name = "testViewAll";
            db.create_view(
                view_name,
                CouchViews::new(
                    view_name,
                    CouchFunc {
                        map: r#"function(doc) {{
                                    emit(doc._id, null);
                            }}"#
                        .to_string(),
                        reduce: None,
                    },
                ),
            )
            .await
            .unwrap();
            let ndoc = db
                .create(json!({
                    "thing": true
                }))
                .await
                .unwrap();
            let ndoc_id = ndoc._id.clone();
            let single_view_name = "testViewSingle";
            db.create_view(
                single_view_name,
                CouchViews::new(
                    single_view_name,
                    CouchFunc {
                        map: format!(
                            r#"function(doc) {{
                                    if(doc._id === "{}") {{
                                        emit(doc._id, null);
                                    }}
                            }}"#,
                            ndoc_id
                        )
                        .to_string(),
                        reduce: None,
                    },
                ),
            )
            .await
            .unwrap();

            let query_result = db.query(view_name, view_name, None).await;

            // executing 'all' view without any params should result in 2 and 1 entries, respectively
            assert_eq!(query_result.unwrap().rows.len(), 2);
            assert_eq!(
                db.query(single_view_name, single_view_name, None)
                    .await
                    .unwrap()
                    .rows
                    .len(),
                1
            );
            // executing 'all' view with default params should result in 2 and 1 entries, respectively
            assert_eq!(
                db.query(view_name, view_name, Some(QueryParams::default()))
                    .await
                    .unwrap()
                    .rows
                    .len(),
                2
            );
            assert_eq!(
                db.query(single_view_name, single_view_name, Some(QueryParams::default()))
                    .await
                    .unwrap()
                    .rows
                    .len(),
                1
            );

            assert!(db.remove(ndoc).await);
            assert!(db.remove(doc).await);

            teardown(client, dbname).await;
        }

        #[tokio::test]
        async fn should_get_many_all_documents_with_keys() {
            let dbname = "should_get_many_all_documents_with_keys";
            let (client, db, docs) = setup_multiple(dbname, 4).await;
            let doc = docs.get(0).unwrap();

            let mut params1 = QueryParams::default();
            params1.key = Some(doc._id.clone());
            let mut params2 = QueryParams::default();
            params2.include_docs = Some(true);
            let mut params3 = QueryParams::default();

            let params = vec![params1, params2, params3];

            let collections = db.query_many_all_docs(QueriesParams::new(params)).await.unwrap();
            assert_eq!(collections.len(), 3);
            assert_eq!(collections.get(0).unwrap().rows.len(), 1);
            // first result has no docs and only 1 row
            assert!(collections.get(0).unwrap().rows.get(0).unwrap().doc.is_none());
            // second result has 4 rows with docs
            assert_eq!(collections.get(1).unwrap().rows.len(), 4);
            assert!(collections.get(1).unwrap().rows.get(0).unwrap().doc.is_some());
            // third result has 4 rows without docs
            assert_eq!(collections.get(2).unwrap().rows.len(), 4);
            assert!(collections.get(2).unwrap().rows.get(0).unwrap().doc.is_none());

            for doc in docs.into_iter() {
                assert!(db.remove(doc).await);
            }

            teardown(client, dbname).await;
        }

        #[tokio::test]
        async fn should_bulk_insert_and_get_many_docs() {
            let (client, db, _doc) = setup("should_bulk_insert_and_get_many_docs").await;
            let docs: Vec<Value> = (0..2000)
                .map(|idx| {
                    json!({
                        "_id": format!("bd_{}", idx),
                        "count": idx,
                    })
                })
                .collect();

            db.bulk_docs(docs).await.expect("should insert 2000 documents");

            // Create a sender and receiver channel pair
            let (tx, mut rx): (Sender<DocumentCollection>, Receiver<DocumentCollection>) = mpsc::channel(1000);

            // Spawn a separate thread to retrieve the batches from Couch
            let t = tokio::spawn(async move {
                db.get_all_batched(tx, 0, 0).await.expect("can not launch batch fetch");
            });

            let mut retrieved = 0;
            while let Some(all_docs) = rx.recv().await {
                retrieved += all_docs.total_rows;
            }

            // 2001 == 2000 we created with bulk_docs + 1 that is created by setup()
            assert_eq!(retrieved, 2001);

            // Wait for the spawned task to finish (should be done by now).
            t.await.unwrap();
            teardown(client, "should_bulk_insert_and_get_many_docs").await;
        }
    }
}
