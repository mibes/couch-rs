use std::error;
use std::fmt;

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct CouchError {
    pub status: reqwest::StatusCode,
    pub message: String,
}

impl CouchError {
    pub fn new(message: String, status: reqwest::StatusCode) -> CouchError {
        CouchError {
            message,
            status,
        }
    }
}

impl fmt::Display for CouchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
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
            status: err.status().unwrap_or(reqwest::StatusCode::NOT_IMPLEMENTED),
            message: err.to_string(),
        }
    }
}

impl std::convert::From<serde_json::Error> for CouchError {
    fn from(err: serde_json::Error) -> Self {
        CouchError {
            status: reqwest::StatusCode::NOT_IMPLEMENTED,
            message: err.to_string(),
        }
    }
}

impl std::convert::From<url::ParseError> for CouchError {
    fn from(err: url::ParseError) -> Self {
        CouchError {
            status: reqwest::StatusCode::NOT_IMPLEMENTED,
            message: err.to_string(),
        }
    }
}

