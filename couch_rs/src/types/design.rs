use serde::{Deserialize, Serialize};

use crate::error::{CouchError, CouchResult, ErrorMessage};

/// Design document created abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct DesignCreated {
    pub result: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>,
}

#[derive(PartialEq)]
pub enum Status {
    Created,
    NotCreated,
}

impl DesignCreated {
    pub fn status(&self) -> CouchResult<Status> {
        if let Some(result) = &self.result {
            if result == "created" {
                Ok(Status::Created)
            } else {
                Ok(Status::NotCreated)
            }
        } else if let Some(err) = &self.error {
            Err(CouchError::CreateDesignFailed(ErrorMessage {
                message: err.clone(),
                upstream: None,
            }))
        } else {
            Err(CouchError::CreateDesignFailed(ErrorMessage {
                message: "DesignCreated did neither contain a 'result' nor an 'error' field as expected".to_string(),
                upstream: None,
            }))
        }
    }
}
