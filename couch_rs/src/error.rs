use std::{error, fmt, rc::Rc};

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub enum CouchError {
    /// A CouchDB operation failed, typically indicated by a specific HTTP error status that was returned.
    OperationFailed(ErrorDetails),
    /// Parsing of a JSON document failed.
    InvalidJson(ErrorMessage),
    /// The provided url is invalid.
    MalformedUrl(ErrorMessage),
}

#[derive(Debug, Clone)]
pub struct ErrorDetails {
    /// Some (bulk) transaction might return an id as part of the error
    pub id: Option<String>,
    /// HTTP Status Code
    pub status: reqwest::StatusCode,
    /// Detailed error message
    pub message: String,
    upstream: Option<UpstreamError>,
}

#[derive(Debug, Clone)]
pub struct ErrorMessage {
    /// Detailed error message
    pub message: String,
    upstream: Option<UpstreamError>,
}

type UpstreamError = Rc<dyn error::Error + 'static>;
pub type CouchResult<T> = Result<T, CouchError>;

impl CouchError {
    pub fn new(message: String, status: reqwest::StatusCode) -> CouchError {
        CouchError::OperationFailed(ErrorDetails {
            id: None,
            message,
            status,
            upstream: None,
        })
    }

    pub fn new_with_id(id: Option<String>, message: String, status: reqwest::StatusCode) -> CouchError {
        CouchError::OperationFailed(ErrorDetails {
            id,
            message,
            status,
            upstream: None,
        })
    }

    pub fn is_not_found(&self) -> bool {
        self.status() == Some(reqwest::StatusCode::NOT_FOUND)
    }

    pub fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            CouchError::OperationFailed(details) => Some(details.status),
            _ => None,
        }
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
        match self {
            CouchError::OperationFailed(details) => {
                if let Some(id) = &details.id {
                    write!(f, "{} -> {}: {}", id, details.status, details.message)
                } else {
                    write!(f, "{}: {}", details.status, details.message)
                }
            }
            CouchError::InvalidJson(err) => write!(f, "{}", err.message),
            CouchError::MalformedUrl(err) => write!(f, "{}", err.message),
        }
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CouchError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        match self {
            CouchError::OperationFailed(details) => details.upstream.as_deref(),
            CouchError::InvalidJson(err) => err.upstream.as_deref(),
            CouchError::MalformedUrl(message) => message.upstream.as_deref(),
        }
    }
}

impl std::convert::From<reqwest::Error> for CouchError {
    fn from(err: reqwest::Error) -> Self {
        CouchError::OperationFailed(ErrorDetails {
            id: None,
            status: err.status().unwrap_or(reqwest::StatusCode::NOT_IMPLEMENTED),
            message: err.to_string(),
            upstream: Some(Rc::new(err)),
        })
    }
}

impl std::convert::From<serde_json::Error> for CouchError {
    fn from(err: serde_json::Error) -> Self {
        CouchError::InvalidJson(ErrorMessage {
            message: err.to_string(),
            upstream: Some(Rc::new(err)),
        })
    }
}

impl std::convert::From<url::ParseError> for CouchError {
    fn from(err: url::ParseError) -> Self {
        CouchError::MalformedUrl(ErrorMessage {
            message: err.to_string(),
            upstream: Some(Rc::new(err)),
        })
    }
}
