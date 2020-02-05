extern crate sofa;

use std::time::SystemTime;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use sofa::document::DocumentCollection;
use std::fs::File;
use std::io::prelude::*;

const DB_HOST: &'static str = "http://127.0.0.1:5984";
const TEST_DB: &'static str = "test_db";

#[tokio::main]
async fn main() {
    println!("Connecting...");
    let now = SystemTime::now();

    // Create a sender and receiver channel pair
    let (tx, rx): (Sender<DocumentCollection>, Receiver<DocumentCollection>) = mpsc::channel();

    // Spawn a separate thread to retrieve the batches from Couch
    let t = tokio::task::spawn_blocking(move || {
        let client = sofa::Client::new_with_timeout(DB_HOST, 120).unwrap();
        let db = client.db(TEST_DB).unwrap();

        db.get_all_batched(tx, 0, 0);
    });

    // Open a file for writing
    let mut file = File::create("test_db.json").unwrap();

    // Loop until the receiving channel is closed.
    loop {
        match rx.recv() {
            Ok(all_docs) => {
                println!("Received {} docs", all_docs.total_rows);

                // unmarshal the documents and write them to a file.
                // (there is probably a more efficient way of doing this...)
                for row in all_docs.rows {
                    file.write_all(serde_json::to_string(&row.doc).unwrap().as_bytes()).unwrap();
                }
            }
            Err(_e) => {
                break;
            }
        }
    }

    // Make sure the file is written before exiting
    file.sync_all().unwrap();

    let elapsed = now.elapsed().unwrap_or_default();
    println!("{} ms", elapsed.as_millis());

    // Wait for the spawned task to finish (should be done by now).
    t.await.unwrap();
}
