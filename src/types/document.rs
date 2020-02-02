use serde::{Serialize, Deserialize};

/// String that represents a Document ID in CouchDB
pub type DocumentId = String;

/// DocumentRef<T> is an abstraction over populated/unpopulated data fields
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum DocumentRef<T> {
    Ref(DocumentId),
    Populated(T)
}

/// Abstracted document creation result
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct DocumentCreatedResult {
    pub id: Option<String>,
    pub ok: Option<bool>,
    pub rev: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>
}
