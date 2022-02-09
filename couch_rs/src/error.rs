use std::error;
use std::fmt;

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct CouchError {
    /// Some (bulk) transaction might return an id as part of the error
    pub id: Option<String>,
    /// HTTP Status Code
    pub status: reqwest::StatusCode,
    /// Detailed error message
    pub message: String,
}

pub type CouchResult<T> = Result<T, CouchError>;

impl CouchError {
    pub fn new(message: String, status: reqwest::StatusCode) -> CouchError {
        CouchError {
            id: None,
            message,
            status,
        }
    }

    pub fn new_with_id(id: Option<String>, message: String, status: reqwest::StatusCode) -> CouchError {
        CouchError { id, status, message }
    }

    pub fn is_not_found(&self) -> bool {
        self.status == reqwest::StatusCode::NOT_FOUND
    }
}

pub trait CouchResultExt<T> {
    /// turns an Ok into an Ok(Some), a not-found into an Ok(None), otherwise it will return the error.
    fn into_option(self) -> CouchResult<Option<T>>;
}

impl<T> CouchResultExt<T> for CouchResult<T> {
    fn into_option(self) -> CouchResult<Option<T>> {
        match self {
            Ok(r) => Ok(Some(r)),
            Err(err) => {
                if err.is_not_found() {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }
}

impl fmt::Display for CouchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{} -> {}: {}", id, self.status, self.message)
        } else {
            write!(f, "{}: {}", self.status, self.message)
        }
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CouchError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl std::convert::From<reqwest::Error> for CouchError {
    fn from(err: reqwest::Error) -> Self {
        CouchError {
            id: None,
            status: err.status().unwrap_or(reqwest::StatusCode::NOT_IMPLEMENTED),
            message: err.to_string(),
        }
    }
}

impl std::convert::From<serde_json::Error> for CouchError {
    fn from(err: serde_json::Error) -> Self {
        CouchError {
            id: None,
            status: reqwest::StatusCode::NOT_IMPLEMENTED,
            message: err.to_string(),
        }
    }
}

impl std::convert::From<url::ParseError> for CouchError {
    fn from(err: url::ParseError) -> Self {
        CouchError {
            id: None,
            status: reqwest::StatusCode::NOT_IMPLEMENTED,
            message: err.to_string(),
        }
    }
}
