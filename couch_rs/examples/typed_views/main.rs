use couch_rs::document::TypedCouchDocument;
use couch_rs::types::document::DocumentId;
use couch_rs::types::query::QueryParams;
use couch_rs::CouchDocument;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const TEST_DB: &str = "test_db";

#[tokio::main]
async fn main() {
    println!("Connecting...");

    // Prepare the Sofa client
    let client = couch_rs::Client::new_local_test().unwrap();

    // This command gets a reference to an existing database, or it creates a new one when it does
    // not yet exist.
    let db = client.db(TEST_DB).await.unwrap();

    // execute a view that counts items per field value
    // key of the view result: field name
    // value of the view result: integer
    match db.query("countByField", "countByField", None).await {
        Ok(view_collection) => {
            for item in view_collection.rows.into_iter() {
                let field = item.key;
                let value = item.value;
                // view item results are already typed and inferred because of next take_* functions
                take_i32(value);
                take_String(field)
                println!("Result: {} -> {}", field, value);
            }
        }
        Err(e) => {
            println!("Unexpected error: {:?}", e);
        }
    }

    println!("All operations are done")
}

fn take_i32(_param: i32) {}
fn take_String(_param: String) {}
