use crate::types::view::ViewCollection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct QueriesParams {
    queries: Vec<QueryParams>,
}

impl QueriesParams {
    pub fn new(params: Vec<QueryParams>) -> Self {
        QueriesParams { queries: params }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct QueriesCollection {
    pub results: Vec<ViewCollection>,
}

/// Query parameters. You can use the builder paradigm to construct these parameters easily:
/// ```
/// use couch_rs::types::query::QueryParams;
/// let _qp = QueryParams::default().group(true).conflicts(false).start_key("1");
/// ```
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

    pub fn conflicts(mut self, conflicts: bool) -> Self {
        self.conflicts = Some(conflicts);
        self
    }

    pub fn descending(mut self, descending: bool) -> Self {
        self.descending = Some(descending);
        self
    }

    pub fn end_key(mut self, end_key: &str) -> Self {
        self.end_key = Some(end_key.to_string());
        self
    }

    pub fn group(mut self, group: bool) -> Self {
        self.group = Some(group);
        self
    }

    pub fn group_level(mut self, group_level: u32) -> Self {
        self.group_level = Some(group_level);
        self
    }

    pub fn include_docs(mut self, include_docs: bool) -> Self {
        self.include_docs = Some(include_docs);
        self
    }

    pub fn attachments(mut self, attachments: bool) -> Self {
        self.attachments = Some(attachments);
        self
    }

    pub fn att_encoding_info(mut self, att_encoding_info: bool) -> Self {
        self.att_encoding_info = Some(att_encoding_info);
        self
    }

    pub fn inclusive_end(mut self, inclusive_end: bool) -> Self {
        self.inclusive_end = Some(inclusive_end);
        self
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    pub fn keys(mut self, keys: Vec<String>) -> Self {
        self.keys = keys;
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn reduce(mut self, reduce: bool) -> Self {
        self.reduce = Some(reduce);
        self
    }

    pub fn skip(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn sorted(mut self, sorted: bool) -> Self {
        self.sorted = Some(sorted);
        self
    }

    pub fn stable(mut self, stable: bool) -> Self {
        self.stable = Some(stable);
        self
    }

    pub fn start_key(mut self, start_key: &str) -> Self {
        self.start_key = Some(start_key.to_string());
        self
    }

    pub fn start_key_doc_id(mut self, start_key_doc_id: &str) -> Self {
        self.start_key_doc_id = Some(start_key_doc_id.to_string());
        self
    }

    pub fn update(mut self, update: &str) -> Self {
        self.update = Some(update.to_string());
        self
    }

    pub fn update_seq(mut self, update_seq: bool) -> Self {
        self.update_seq = Some(update_seq);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_params_builder_paradigm() {
        let qp = QueryParams::default().group(true).conflicts(false).start_key("1");
        assert_eq!(qp.group, Some(true));
        assert_eq!(qp.start_key, Some("1".to_string()));
    }
}
