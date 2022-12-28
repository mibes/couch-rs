use super::*;
use document::DocumentId;
use find::SortSpec;
use serde::{Deserialize, Serialize};

/// Index fields abstraction
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct IndexFields {
    pub fields: Vec<SortSpec>,
}

impl IndexFields {
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
    pub index_type: IndexType,
    pub def: IndexFields,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum IndexType {
    Json,
    Text,
}

/// Database index list abstraction
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct DatabaseIndexList {
    pub total_rows: u32,
    pub indexes: Vec<Index>,
}
