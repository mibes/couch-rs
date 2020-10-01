use crate::document::TypedCouchDocument;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(bound(deserialize = "T: TypedCouchDocument"))]
pub struct ViewCollection<K: DeserializeOwned, V: DeserializeOwned, T: TypedCouchDocument> {
    pub offset: Option<u32>,
    pub rows: Vec<ViewItem<K, V, T>>,
    pub total_rows: Option<u32>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(bound(deserialize = "T: TypedCouchDocument"))]
pub struct ViewItem<K: DeserializeOwned, V: DeserializeOwned, T: TypedCouchDocument> {
    pub key: K,
    pub value: V,
    pub id: Option<String>,
    // docs field, populated if query was ran with 'include_docs'
    pub doc: Option<T>,
}

/// CouchViews can be used to create one of more views in a particular design document.
#[derive(Serialize)]
pub struct CouchViews {
    views: HashMap<String, CouchFunc>,
    language: String,
}

/// Constructs a CouchDB View Function. See
/// [defining-a-view](https://docs.couchdb.org/en/stable/ddocs/views/nosql.html#defining-a-view) for
/// details.
///
/// ```
/// use couch_rs::types::view::CouchFunc;
/// let couch_func = CouchFunc {
///     map: "function (doc) { if (doc.CLIP == true) { emit(doc.CLIP); } }".to_string(),
///     reduce: None,
/// };
/// ```
#[derive(Serialize)]
pub struct CouchFunc {
    pub map: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CouchUpdate {
    updates: HashMap<String, String>,
}

impl CouchViews {
    pub fn new(view_name: &str, func: CouchFunc) -> Self {
        let mut couch_views = CouchViews::default();
        couch_views.add(view_name, func);
        couch_views
    }

    pub fn add(&mut self, name: &str, func: CouchFunc) {
        self.views.insert(name.to_string(), func);
    }
}

impl Default for CouchViews {
    fn default() -> Self {
        CouchViews {
            views: HashMap::new(),
            language: "javascript".to_string(),
        }
    }
}

impl CouchFunc {
    pub fn new(map: &str, reduce: Option<&str>) -> Self {
        CouchFunc {
            map: map.to_string(),
            reduce: reduce.map(|r| r.to_string()),
        }
    }
}

impl Into<serde_json::Value> for CouchViews {
    fn into(self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

impl Into<serde_json::Value> for CouchFunc {
    fn into(self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

impl CouchUpdate {
    pub fn new(func_name: &str, func: &str) -> Self {
        let mut update = CouchUpdate {
            updates: HashMap::new(),
        };
        update.add(func_name, func);
        update
    }

    pub fn add(&mut self, name: &str, func: &str) {
        self.updates.insert(name.to_string(), func.to_string());
    }
}

impl Into<serde_json::Value> for CouchUpdate {
    fn into(self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}
