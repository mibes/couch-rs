use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Event {
    Change(ChangeEvent),
    Finished(FinishedEvent),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChangeEvent {
    pub seq: serde_json::Value,
    pub id: String,
    pub changes: Vec<Change>,

    #[serde(default)]
    pub deleted: bool,

    #[serde(default)]
    pub doc: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Change {
    pub rev: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FinishedEvent {
    pub last_seq: serde_json::Value,
    pub pending: Option<u64>, // not available on CouchDB 1.0
}
