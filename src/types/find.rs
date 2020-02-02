use serde_json::{Value};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Sort direction abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum SortDirection {
    Desc,
    Asc
}

impl From<String> for SortDirection {
    fn from(original: String) -> SortDirection {
        match original.as_ref() {
            "desc" => SortDirection::Desc,
            "asc" | _ => SortDirection::Asc
        }
    }
}

/// Sort spec content abstraction
pub type SortSpecContent = HashMap<String, String>;

/// Sort spec abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum SortSpec {
    Simple(String),
    Complex(SortSpecContent)
}

/// Index spec abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum IndexSpec {
    DesignDocument(String),
    IndexName((String, String))
}

/// Find query abstraction
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FindQuery {
    pub selector: Value,
    pub limit: Option<u64>,
    pub skip: Option<u64>,
    pub sort: Option<SortSpec>,
    pub fields: Option<Vec<String>>,
    pub use_index: Option<IndexSpec>
}

/// Find result abstraction
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct FindResult {
    pub docs: Option<Vec<Value>>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>,
}

//todo: include status on structs

/// Explain result abstraction
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ExplainResult {
    pub dbname: String,
    pub index: IndexSpec,
    pub selector: Value,
    pub opts: Value,
    pub limit: u32,
    pub skip: u64,
    pub fields: Vec<String>,
    pub range: Value
}
