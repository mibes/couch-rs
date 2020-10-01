use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::ops::{Index, IndexMut};

pub trait TypedCouchDocument: DeserializeOwned + Serialize + Sized {
    fn get_id(&self) -> Cow<str>;
    fn get_rev(&self) -> Cow<str>;
    fn set_rev(&mut self, rev: &str);
    fn set_id(&mut self, rev: &str);
    fn merge(&mut self, other: Self);
}

impl TypedCouchDocument for Value {
    fn get_id(&self) -> Cow<str> {
        let id: String = json_extr!(self["_id"]);
        Cow::from(id)
    }

    fn get_rev(&self) -> Cow<str> {
        let rev: String = json_extr!(self["_rev"]);
        Cow::from(rev)
    }

    fn set_id(&mut self, id: &str) {
        if let Some(o) = self.as_object_mut() {
            o.insert("_id".to_string(), Value::from(id));
        }
    }

    fn set_rev(&mut self, rev: &str) {
        if let Some(o) = self.as_object_mut() {
            o.insert("_rev".to_string(), Value::from(rev));
        }
    }

    fn merge(&mut self, other: Self) {
        self.set_id(&other.get_id());
        self.set_rev(&other.get_rev());
    }
}

/// Memory-optimized, iterable document collection, mostly returned in calls
/// that involve multiple documents results Can target a specific index through
/// implementation of `Index` and `IndexMut`
#[derive(PartialEq, Debug, Clone)]
pub struct DocumentCollection<T: TypedCouchDocument> {
    pub offset: Option<u32>,
    pub rows: Vec<T>,
    pub total_rows: u32,
    pub bookmark: Option<String>,
}

impl<T: TypedCouchDocument> Default for DocumentCollection<T> {
    fn default() -> Self {
        DocumentCollection {
            offset: None,
            rows: vec![],
            total_rows: 0,
            bookmark: None,
        }
    }
}

impl<T: TypedCouchDocument> DocumentCollection<T> {
    pub fn new(doc: Value) -> DocumentCollection<T> {
        let rows: Vec<Value> = json_extr!(doc["rows"]);
        let items: Vec<T> = rows
            .into_iter()
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
                let document: T = json_extr!(d["doc"]);
                document
            })
            .collect();

        DocumentCollection {
            offset: json_extr!(doc["offset"]),
            total_rows: items.len() as u32,
            rows: items,
            bookmark: Option::None,
        }
    }

    pub fn new_from_documents(docs: Vec<T>, bookmark: Option<String>) -> DocumentCollection<T> {
        let len = docs.len() as u32;
        DocumentCollection {
            offset: Some(0),
            total_rows: len,
            rows: docs,
            bookmark,
        }
    }

    pub fn new_from_values(docs: Vec<Value>, bookmark: Option<String>) -> DocumentCollection<T> {
        let len = docs.len() as u32;

        DocumentCollection {
            offset: Some(0),
            total_rows: len,
            rows: docs
                .into_iter()
                .filter_map(|d| serde_json::from_value::<T>(d).ok())
                .collect(),
            bookmark,
        }
    }

    /// Returns raw JSON data from documents
    pub fn get_data(&self) -> &Vec<T> {
        &self.rows
    }
}

impl<T: TypedCouchDocument> Index<usize> for DocumentCollection<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self.rows.get(index).unwrap()
    }
}

impl<T: TypedCouchDocument> IndexMut<usize> for DocumentCollection<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.rows.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate as couch_rs;
    use crate::document::TypedCouchDocument;
    use couch_rs_derive::CouchDocument;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, CouchDocument, Debug, Default)]
    struct TestDocument {
        #[serde(skip_serializing_if = "String::is_empty")]
        pub _id: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub _rev: String,
    }

    #[test]
    fn test_derive_couch_document() {
        let doc = TestDocument {
            _id: "1".to_string(),
            _rev: "2".to_string(),
        };
        let id = doc.get_id();
        let rev = doc.get_rev();
        assert_eq!(id, "1");
        assert_eq!(rev, "2");
    }
}