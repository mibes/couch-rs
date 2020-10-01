use couch_rs::error::CouchResult;
use couch_rs::types::view::RawViewCollection;

const TEST_DB: &str = "test_db";

#[tokio::main]
async fn main() -> CouchResult<()> {
    println!("Connecting...");

    // Prepare the Sofa client
    let client = couch_rs::Client::new_local_test().unwrap();

    // This command gets a reference to an existing database, or it creates a new one when it does
    // not yet exist.
    let db = client.db(TEST_DB).await.unwrap();

    // execute a view that counts items per field value
    // key of the view result: field name
    // value of the view result: integer
    // doc: Value

    let result: RawViewCollection<String, i32> = db.query("countByField", "countByField", None).await?;
    for item in result.rows.into_iter() {
        let field = item.key;
        let count = item.value;
        // view item results are already typed
        println!("fieldname is {} and count is {}", field, count);
    }
    println!("All operations are done");

    Ok(())
}
