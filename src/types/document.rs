use crate::error::{CouchError, CouchResult};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// String that represents a Document ID in CouchDB
pub type DocumentId = String;

/// DocumentRef<T> is an abstraction over populated/unpopulated data fields
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum DocumentRef<T> {
    Ref(DocumentId),
    Populated(T),
}

/// Abstracted document creation response
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct DocumentCreatedResponse {
    pub id: Option<String>,
    pub ok: Option<bool>,
    pub rev: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>,
}

/// Abstracted document creation result
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct DocumentCreatedDetails {
    pub id: Option<String>,
    pub rev: Option<String>,
}

impl From<DocumentCreatedResponse> for DocumentCreatedResult {
    fn from(response: DocumentCreatedResponse) -> Self {
        if let Some(error) = response.error {
            let status_code = match error.as_str() {
                "forbidden" => StatusCode::FORBIDDEN,
                "unauthorized" => StatusCode::UNAUTHORIZED,
                "conflict" => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            Err(CouchError::new(response.reason.unwrap_or_default(), status_code))
        } else {
            Ok(DocumentCreatedDetails {
                id: response.id,
                rev: response.rev,
            })
        }
    }
}

pub type DocumentCreatedResult = CouchResult<DocumentCreatedDetails>;
