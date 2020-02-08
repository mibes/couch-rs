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

extern crate sofa;

use serde_json::{json, Value};

/// Update DB_HOST to point to your running Couch instance
const DB_HOST: &'static str = "http://localhost:5984";
const TEST_DB: &'static str = "test_db";

/// test_docs generates a bunch of documents that can be used in the _bulk_docs operation.
fn test_docs(amount: i32) -> Vec<Value> {
    let mut result: Vec<Value> = vec![];

    for _i in 0..amount {
        result.push(json!({"name": "Marcel"}))
    }

    result
}

#[tokio::main]
async fn main() {
    println!("Connecting...");

    // Prepare the Sofa client
    let client = sofa::Client::new(DB_HOST).unwrap();
    let mut db_initialized = false;

    // This command gets a reference to an existing database, or it creates a new one when it does
    // not yet exist.
    let db = client.db(TEST_DB).await.unwrap();

    // List the existing databases. The db_initialized is superfluous, since we just created it in
    // the previous step. It is for educational purposes only.
    match client.list_dbs().await {
        Ok(dbs) => {
            println!("Existing databases:");
            for db in dbs {
                println!("Couch DB {}", db);

                if db == TEST_DB {
                    db_initialized = true;
                }
            }
        }
        Err(err) => panic!("Oops: {:?}", err),
    }

    let mut first_doc_id: Option<String> = None;

    if db_initialized {
        // let's add some docs
        match db.bulk_docs(test_docs(100)).await {
            Ok(resp) => {
                println!("Bulk docs completed");

                first_doc_id = resp.first().unwrap().clone().id;

                for r in resp {
                    println!("Id: {}, OK?: {}", r.id.unwrap_or("--".to_string()), r.ok.unwrap_or(false))
                }
            }
            Err(err) => println!("Oops: {:?}", err),
        }
    }

    println!("---");

    if first_doc_id.is_some() {
        // we have an id of the first document we've just inserted
        match db.get(first_doc_id.unwrap()).await {
            Ok(doc) => { println!("First document: {}", doc.get_data().to_string()) }
            Err(err) => println!("Oops: {:?}", err),
        }
    }

    println!("All operations are done")
}
