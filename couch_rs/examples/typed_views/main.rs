use serde_json::Value;

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
    match db
        .query::<String, i32, Value>("countByField", "countByField", None)
        .await
    {
        Ok(view_collection) => {
            for item in view_collection.rows.into_iter() {
                let field: String = item.key;
                let value: i32 = item.value;
                // view item results are already typed
                take_i32(value);
                take_string(field);
            }
        }
        Err(e) => {
            println!("Unexpected error: {:?}", e);
        }
    }

    println!("All operations are done")
}

fn take_i32(_param: i32) {}
fn take_string(_param: String) {}
