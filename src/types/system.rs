use serde::{Serialize, Deserialize};

/// Couch vendor abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct CouchVendor {
    pub name: String,
    pub version: Option<String>
}

/// Couch status abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct CouchStatus {
    pub couchdb: String,
    pub git_sha: Option<String>,
    pub uuid: Option<String>,
    pub version: String,
    pub vendor: CouchVendor
}

/// Couch response abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct CouchResponse {
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub reason: Option<String>
}
