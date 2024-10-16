use super::{document, find};
use document::DocumentId;
use find::SortSpec;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Index fields abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct IndexFields {
    pub fields: Vec<SortSpec>,
}

impl IndexFields {
    #[must_use]
    pub fn new(fields: Vec<SortSpec>) -> IndexFields {
        IndexFields { fields }
    }
}

/// Index abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Index {
    pub ddoc: Option<DocumentId>,
    pub name: String,
    #[serde(rename = "type")]
    pub index_type: Option<IndexType>,
    pub def: IndexFields,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum IndexType {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "special")]
    Special, // reserved for primary index
}

impl fmt::Display for IndexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexType::Json => write!(f, "json"),
            IndexType::Text => write!(f, "text"),
            IndexType::Special => write!(f, "special"),
        }
    }
}

/// Database index list abstraction
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct DatabaseIndexList {
    pub total_rows: u32,
    pub indexes: Vec<Index>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteIndexResponse {
    pub ok: bool,
}
