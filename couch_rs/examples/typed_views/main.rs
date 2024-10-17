use couch_rs::{
    error::CouchResult,
    types::view::{CouchFunc, CouchViews, RawViewCollection},
};
use serde_json::json;

const TEST_DB: &str = "view_db";

#[tokio::main]
async fn main() -> CouchResult<()> {
    let client = couch_rs::Client::new_local_test()?;
    let db = client.db(TEST_DB).await?;

    let mut doc = json!({
        "_id": "jdoe",
        "first_name": "John",
        "last_name": "Doe",
        "funny": true
    });

    db.create(&mut doc).await?;

    let couch_func = CouchFunc {
        map: "function (doc) { if (doc.funny == true) { emit(doc._id, doc.funny); } }".to_string(),
        reduce: None,
    };

    let couch_views = CouchViews::new("funny_guys", couch_func);
    db.create_view("test_design", couch_views).await?;
    let result: RawViewCollection<String, bool> = db.query("test_design", "funny_guys", None).await?;

    println!("Funny guys:");
    for item in result.rows {
        let id = item.key;
        let is_funny = item.value;
        println!("{id} is funny: {is_funny}");
    }
    Ok(())
}
