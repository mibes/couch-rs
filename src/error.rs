use std::{error, fmt};

/// Custom error type
#[derive(Debug)]
pub struct SofaError {
    err: String
}

impl error::Error for SofaError {
    fn description(&self) -> &str {
        &self.err
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for SofaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.err)
    }
}

impl From<String> for SofaError {
    fn from(s: String) -> SofaError {
        SofaError { err: s }
    }
}

impl From<&'static str> for SofaError {
    fn from(s: &str) -> SofaError {
        SofaError { err: s!(s) }
    }
}
