use crate::error::{CouchError, CouchResult};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// String that represents a Document ID in `CouchDB`
pub type DocumentId = String;

/// `DocumentRef`<T> is an abstraction over populated/unpopulated data fields
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum DocumentRef<T> {
    Ref(DocumentId),
    Populated(T),
}

/// Abstracted document creation response
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub(crate) struct DocumentCreatedResponse {
    /// Document ID
    pub id: Option<String>,
    /// New document revision token. Available if document has saved without errors
    pub rev: Option<String>,
    /// Operation status. Available in case of success
    pub ok: Option<bool>,
    /// Error type. Available if response code is 4xx
    pub error: Option<String>,
    /// Error description. Available if response code is 4xx
    pub reason: Option<String>,
}

/// Abstracted document creation result
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct DocumentCreatedDetails {
    /// Document ID
    pub id: String,
    /// New document revision token.
    pub rev: String,
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

            Err(CouchError::new_with_id(
                response.id,
                response.reason.unwrap_or_default(),
                status_code,
            ))
        } else {
            match (response.id, response.rev) {
                (Some(id), Some(rev)) => Ok(DocumentCreatedDetails { id, rev }),
                (_, _) => Err(CouchError::new(
                    "Unexpected response format".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )),
            }
        }
    }
}

pub type DocumentCreatedResult = CouchResult<DocumentCreatedDetails>;
