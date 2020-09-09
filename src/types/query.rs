use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub descending: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_key_doc_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_level: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_docs: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub att_encoding_info: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inclusive_end: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorted: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_key_doc_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_seq: Option<bool>,
}

impl Default for QueryParams {
    fn default() -> Self {
        QueryParams {
            conflicts: None,
            descending: None,
            end_key: None,
            end_key_doc_id: None,
            group: None,
            group_level: None,
            include_docs: None,
            attachments: None,
            att_encoding_info: None,
            inclusive_end: None,
            key: None,
            keys: vec![],
            limit: None,
            reduce: None,
            skip: None,
            sorted: None,
            stable: None,
            stale: None,
            start_key: None,
            start_key_doc_id: None,
            update: None,
            update_seq: None,
        }
    }
}

impl QueryParams {
    pub fn from_keys(keys: Vec<String>) -> Self {
        QueryParams {
            conflicts: None,
            descending: None,
            end_key: None,
            end_key_doc_id: None,
            group: None,
            group_level: None,
            include_docs: None,
            attachments: None,
            att_encoding_info: None,
            inclusive_end: None,
            key: None,
            keys,
            limit: None,
            reduce: None,
            skip: None,
            sorted: None,
            stable: None,
            stale: None,
            start_key: None,
            start_key_doc_id: None,
            update: None,
            update_seq: None,
        }
    }
}
