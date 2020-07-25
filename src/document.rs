use serde_json::Value;
use std::ops::{Index, IndexMut};
use serde::{Serialize, Deserialize};
use crate::types::document::{DocumentId};
use crate::database::Database;

/// Document abstracts the handling of JSON values and provides direct access
/// and casting to the fields of your documents You can get access to the
/// fields via the implementation of the `Index` and `IndexMut` traits
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Document {
    /// Document ID, provided by CouchDB
    #[serde(skip_serializing)]
    pub _id: DocumentId,

    /// Document Revision, provided by CouchDB, helps negotiating conflicts
    #[serde(skip_serializing)]
    pub _rev: String,

    doc: Value,
}

impl Document {
    pub fn new(doc: Value) -> Document {
        Document {
            _id: json_extr!(doc["_id"]),
            _rev: json_extr!(doc["_rev"]),
            doc,
        }
    }

    /// Returns all document's keys
    pub fn get_keys(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();

        if let Some(obj) = self.doc.as_object() {
            for (k, _) in obj.into_iter() {
                ret.push(k.clone());
            }
        }

        ret
    }

    /// Returns raw JSON data from document
    pub fn get_data(&self) -> Value {
        self.doc.clone()
    }

    /// Merges this document with a raw JSON value, useful to update data with
    /// a payload
    pub fn merge(&mut self, doc: Value) -> &Self {
        if let Some(obj) = doc.as_object() {
            for (k, v) in obj.into_iter() {
                match k.as_str() {
                    "_id" | "_rev" => {
                        continue;
                    }
                    _ => {
                        self[k] = v.clone();
                    }
                }
            }
        }

        self
    }

    /// Recursively populates field (must be an array of IDs from another
    /// database) with provided database documents
    pub async fn populate(&mut self, field: &str, db: Database) -> &Self {
        let val = &self[field].clone();
        if *val == Value::Null {
            return self;
        }

        let ids = val.as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|v| s!(v.as_str().unwrap_or("")))
            .collect();

        let data = db.get_bulk(ids).await.map(|docs| docs.get_data());

        match data {
            Ok(data) => {
                self[field] = data.into_iter()
                    .filter_map(|d: Value| {
                        let did = match d["_id"].as_str() {
                            Some(did) => did,
                            None => return None,
                        };

                        if val[did] != Value::Null {
                            Some(d.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
            }
            Err(_) => {
                return self;
            }
        }

        self
    }
}

impl<I> Index<I> for Document
where
    I: serde_json::value::Index,
{
    type Output = Value;

    fn index(&self, index: I) -> &Value {
        &self.doc[index]
    }
}

impl<I> IndexMut<I> for Document
where
    I: serde_json::value::Index,
{
    fn index_mut(&mut self, index: I) -> &mut Value {
        &mut self.doc[index]
    }
}

/// Used inside a `DocumentCollection`, to wrap the document itself and
/// facilitate lookups by Document ID.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DocumentCollectionItem {
    pub id: DocumentId,
    pub doc: Document,
}

impl DocumentCollectionItem {
    pub fn new(doc: Document) -> DocumentCollectionItem {
        let id = doc._id.clone();
        DocumentCollectionItem { doc, id }
    }
}

/// Memory-optimized, iterable document collection, mostly returned in calls
/// that involve multiple documents results Can target a specific index through
/// implementation of `Index` and `IndexMut`
#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DocumentCollection {
    pub offset: Option<u32>,
    pub rows: Vec<DocumentCollectionItem>,
    pub total_rows: u32,
    pub bookmark: Option<String>,
}

impl DocumentCollection {
    pub fn new(doc: Value) -> DocumentCollection {
        let rows: Vec<Value> = json_extr!(doc["rows"]);
        let items: Vec<DocumentCollectionItem> = rows.into_iter()
            .filter(|d| {
                let maybe_err: Option<String> = json_extr!(d["error"]);
                if maybe_err.is_some() {
                    // remove errors
                    false
                } else {
                    // Remove _design documents
                    let id: String = json_extr!(d["doc"]["_id"]);
                    !id.starts_with('_')
                }
            })
            .map(|d| {
                let document: Value = json_extr!(d["doc"]);
                DocumentCollectionItem::new(Document::new(document))
            })
            .collect();

        DocumentCollection {
            offset: json_extr!(doc["offset"]),
            total_rows: items.len() as u32,
            rows: items,
            bookmark: Option::None,
        }
    }

    pub fn new_from_documents(docs: Vec<Document>, bookmark: Option<String>) -> DocumentCollection {
        let len = docs.len() as u32;

        DocumentCollection {
            offset: Some(0),
            total_rows: len,
            rows: docs.into_iter().map(DocumentCollectionItem::new).collect(),
            bookmark,
        }
    }

    /// Returns raw JSON data from documents
    pub fn get_data(&self) -> Vec<Value> {
        self.rows.iter().map(|doc_item| doc_item.doc.get_data()).collect()
    }
}

impl Index<usize> for DocumentCollection {
    type Output = DocumentCollectionItem;

    fn index(&self, index: usize) -> &DocumentCollectionItem {
        &self.rows.get(index).unwrap()
    }
}

impl IndexMut<usize> for DocumentCollection {
    fn index_mut(&mut self, index: usize) -> &mut DocumentCollectionItem {
        self.rows.get_mut(index).unwrap()
    }
}
