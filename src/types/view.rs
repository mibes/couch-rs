use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ViewCollection {
    pub offset: Option<u32>,
    pub rows: Vec<ViewItem>,
    pub total_rows: Option<u32>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ViewItem {
    pub key: String,
    pub value: Value,
    pub id: Option<String>,
    // docs field, populated if query was ran with 'include_docs'
    pub doc: Option<Value>,
}
