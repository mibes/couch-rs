extern crate sofa;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sofa::types::document::DocumentId;

/// Update DB_HOST to point to your running Couch instance
const DB_HOST: &str = "http://admin:password@localhost:5984";
const TEST_DB: &str = "test_db";

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TestDoc {
    /// _ids are are the only unique enforced value within CouchDB so you might as well make use of this.
    /// CouchDB stores its documents in a B+ tree. Each additional or updated document is stored as
    /// a leaf node, and may require re-writing intermediary and parent nodes. You may be able to take
    /// advantage of sequencing your own ids more effectively than the automatically generated ids if
    /// you can arrange them to be sequential yourself. (https://docs.couchdb.org/en/stable/best-practices/documents.html)
    pub _id: DocumentId,

    /// Document Revision, provided by CouchDB, helps negotiating conflicts
    #[serde(skip_serializing)]
    pub _rev: String,

    pub first_name: String,
    pub last_name: String,
}

#[tokio::main]
async fn main() {
    println!("Connecting...");

    // Prepare the Sofa client
    let client = sofa::Client::new(DB_HOST).unwrap();

    // This command gets a reference to an existing database, or it creates a new one when it does
    // not yet exist.
    let db = client.db(TEST_DB).await.unwrap();

    let td = TestDoc {
        _id: "1234".to_string(),
        _rev: "".to_string(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
    };

    // check if the document already exists
    match db.get("1234".to_string()).await {
        Ok(existing) => {
            println!("Document has been previously created with Rev: {}", existing._rev);
            let e: TestDoc = serde_json::from_value(existing.get_data()).unwrap();
            println!("Name: {} {}", e.first_name, e.last_name);
        }
        Err(e) => {
            match e.status {
                StatusCode::NOT_FOUND => {
                    // create the document
                    match db.create(serde_json::to_value(td).unwrap()).await {
                        Ok(r) => println!("Document was created with ID: {} and Rev: {}", r._id, r._rev),
                        Err(err) => println!("Oops: {:?}", err),
                    }
                }
                _ => {
                    println!("Unexpected error: {:?}", e);
                }
            }
        }
    }

    println!("All operations are done")
}
