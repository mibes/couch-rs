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
use couch_rs::types::query::{QueriesParams, QueryParams};
use serde_json::{json, Value};
use std::error::Error;

/// Update DB_HOST to point to your running Couch instance
const DB_HOST: &str = "http://admin:password@localhost:5984";
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
    let client = couch_rs::Client::new(DB_HOST).unwrap();

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
    match db.bulk_docs(test_docs(100)).await {
        Ok(resp) => {
            println!("Bulk docs completed");

            for r in resp {
                println!(
                    "Id: {}, OK?: {}",
                    r.id.unwrap_or_else(|| "--".to_string()),
                    r.ok.unwrap_or(false)
                )
            }
        }
        Err(err) => println!("Oops: {:?}", err),
    }

    println!("--- Finding ---");

    let find_all = FindQuery::find_all();
    let docs = db.find(&find_all).await?;
    if let Some(row) = docs.rows.get(0) {
        println!("First document: {}", row.doc.get_data().to_string())
    }

    // imagine we have a database (e.g. vehicles) with multiple documents of different types; e.g. cars, planes and boats
    // document IDs have been generated taking this into account, so cars have IDs starting with "car:",
    // planes have IDs starting with "plane:", and boats have IDs starting with "boat:"
    //
    // let's query for all cars and all boats, sending just 1 request
    let mut cars = QueryParams::default();
    cars.start_key = Some("car".to_string());
    cars.end_key = Some("car:\u{fff0}".to_string());
    match db.query_many_all_docs(QueriesParams::new(vec![cars])).await {
        Ok(mut collections) => {
            println!("Succeeded querying for cars and boats");
            let mut collections = collections.iter_mut();
            let car_collection = collections.next().unwrap();
            println!("Retrieved cars {:?}", car_collection);
            let boat_collection = collections.next().unwrap();
            println!("Retrieved boats {:?}", boat_collection);
        }
        Err(err) => println!("Oops: {:?}", err),
    }

    println!("All operations are done");
    Ok(())
}
