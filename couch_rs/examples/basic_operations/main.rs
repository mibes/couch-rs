/// This example demonstrates some basic Couch operations: connecting, listing databases and
/// inserting some documents in bulk.
///
/// The easiest way to get this example to work, is to connect it to a running CouchDB Docker
/// container:
///
/// ```
/// docker run --rm -p5984:5984 couchdb:2.3.1
/// ```
///
/// Depending on the Docker framework you are using it may listen to "localhost" or to some other
/// automatically assigned IP address. Minikube for example generates a unique IP on start-up. You
/// can obtain it with: `minikube ip`
use couch_rs::types::find::FindQuery;
use serde_json::{json, Value};
use std::error::Error;

const TEST_DB: &str = "test_db";

/// test_docs generates a bunch of documents that can be used in the _bulk_docs operation.
fn test_docs(amount: i32) -> Vec<Value> {
    let mut result: Vec<Value> = vec![];

    for _i in 0..amount {
        result.push(json!({"name": "Marcel"}))
    }

    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting...");

    // Prepare the Sofa client
    let client = couch_rs::Client::new_local_test().unwrap();

    // This command gets a reference to an existing database, or it creates a new one when it does
    // not yet exist.
    let db = client.db(TEST_DB).await.unwrap();
    // List the existing databases. The db_initialized is superfluous, since we just created it in
    // the previous step. It is for educational purposes only.
    let dbs = client.list_dbs().await?;
    let mut db_initialized: bool = false;
    println!("Existing databases:");
    for db in dbs {
        println!("Couch DB {}", db);
        if db == TEST_DB {
            db_initialized = true;
        }
    }

    if !db_initialized {
        println!("{} not found", TEST_DB);
        return Ok(());
    }

    println!("--- Creating ---");

    // let's add some docs
    let mut test_docs = test_docs(100);
    match db.bulk_docs(&mut test_docs).await {
        Ok(resp) => {
            println!("Bulk docs completed");

            for r in resp {
                match r {
                    Ok(details) => println!("Id: {}", details.id),
                    Err(err) => println!("Error: {:?}", err),
                }
            }
        }
        Err(err) => println!("Oops: unable to create documents {:?}: {:?}", test_docs, err),
    }

    println!("--- Finding ---");

    let find_all = FindQuery::find_all();
    let docs = db.find::<Value>(&find_all).await?;
    if let Some(row) = docs.rows.first() {
        println!("First document: {}", row)
    }

    println!("All operations are done");
    Ok(())
}
