use couch_rs::document::DocumentCollection;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::time::SystemTime;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

const TEST_DB: &str = "test_db";

#[tokio::main]
async fn main() {
    println!("Connecting...");
    let now = SystemTime::now();

    // Create a sender and receiver channel pair
    let (tx, mut rx): (Sender<DocumentCollection<Value>>, Receiver<DocumentCollection<Value>>) = mpsc::channel(100);

    // Spawn a separate thread to retrieve the batches from Couch
    let t = tokio::spawn(async move {
        let client = couch_rs::Client::new_local_test().unwrap();
        let db = client.db(TEST_DB).await.unwrap();

        if let Err(err) = db.get_all_batched(tx, 0, 0).await {
            println!("error during batch read: {:?}", err);
        }
    });

    // Open a file for writing
    let mut file = File::create("test_db.json").unwrap();

    // Loop until the receiving channel is closed.
    while let Some(all_docs) = rx.recv().await {
        println!("Received {} docs", all_docs.total_rows);

        // unmarshal the documents and write them to a file.
        // (there is probably a more efficient way of doing this...)
        for row in all_docs.rows {
            file.write_all(serde_json::to_string(&row).unwrap().as_bytes()).unwrap();
        }
    }

    // Make sure the file is written before exiting
    file.sync_all().unwrap();

    let elapsed = now.elapsed().unwrap_or_default();
    println!("{} ms", elapsed.as_millis());

    // Wait for the spawned task to finish (should be done by now).
    t.await.unwrap();
}
