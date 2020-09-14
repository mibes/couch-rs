use serde::{Serialize, Deserialize};
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
}