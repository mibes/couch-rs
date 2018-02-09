use std::collections::HashMap;

use reqwest::StatusCode;

use reqwest::Error;
use serde_json;
use serde_json::{Value, from_reader, to_string};

use ::client::*;
use ::document::*;
use ::types::*;
use ::error::SofaError;

/// Database holds the logic of making operations on a CouchDB Database (sometimes called Collection in other NoSQL flavors such as MongoDB).
#[derive(Debug, Clone)]
pub struct Database {
    _client: Client,
    name: String
}

impl Database {
    pub fn new(name: String, client: Client) -> Database {
        Database {
            _client: client,
            name: name
        }
    }

    fn create_document_path(&self, id: DocumentId) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/");
        result.push_str(&id);
        result
    }

    #[allow(dead_code)]
    fn create_design_path(&self, id: DocumentId) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/_design/");
        result.push_str(&id);
        result
    }

    fn create_compact_path(&self, design_name: &'static str) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/_compact/");
        result.push_str(design_name);
        result
    }

    /// Launches the compact process
    pub fn compact(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_compact");
        let response = self._client.post(path, "".into())
            .send()
            .unwrap();

        match response.status() {
            StatusCode::Accepted => true,
            _ => false
        }
    }

    /// Starts the compaction of all views
    pub fn compact_views(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_view_cleanup");
        let response = self._client.post(path, "".into())
            .send()
            .unwrap();

        match response.status() {
            StatusCode::Accepted => true,
            _ => false
        }
    }

    /// Starts the compaction of a given index
    pub fn compact_index(&self, index: &'static str) -> bool {
        let response = self._client.post(self.create_compact_path(index), "".into())
            .send()
            .unwrap();

        match response.status() {
            StatusCode::Accepted => true,
            _ => false
        }
    }

    /// Checks if a document ID exists
    pub fn exists(&self, id: DocumentId) -> bool {
        let response = self._client.head(self.create_document_path(id), None)
            .send()
            .unwrap();

        match response.status() {
            StatusCode::Ok | StatusCode::NotModified => true,
            _ => false
        }
    }

    /// Gets one document
    pub fn get(&self, id: DocumentId) -> Result<Document, SofaError> {
        let response = self._client.get(self.create_document_path(id), None)
            .send()
            .unwrap();

        Ok(Document::new(from_reader(response).unwrap()))
    }

    /// Gets documents in bulk with provided IDs list
    pub fn get_bulk(&self, ids: Vec<DocumentId>) -> Result<DocumentCollection, Error> {
        self.get_bulk_params(ids, None)
    }

    /// Gets documents in bulk with provided IDs list, with added params. Params description can be found here: Parameters description can be found here: http://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view
    pub fn get_bulk_params(&self, ids: Vec<DocumentId>, params: Option<HashMap<String, String>>) -> Result<DocumentCollection, Error> {
        let mut options;
        if let Some(opts) = params {
            options = opts;
            options.insert(s!("include_docs"), s!("true"));
        } else {
            options = hashmap!{
                s!("include_docs") => s!("true")
            };
        }

        let response = self._client.get(
            self.create_document_path("_all_docs".into()),
            Some(options)
        ).body(to_string(&hashmap!{
            s!("keys") => ids
        }).unwrap())
        .send()
        .unwrap();

        Ok(DocumentCollection::new(from_reader(response).unwrap()))
    }

    /// Gets all the documents in database
    pub fn get_all(&self) -> Result<DocumentCollection, Error> {
        self.get_all_params(None)
    }

    /// Gets all the documents in database, with applied parameters. Parameters description can be found here: http://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view
    pub fn get_all_params(&self, params: Option<HashMap<String, String>>) -> Result<DocumentCollection, Error> {
        let mut options;
        if let Some(opts) = params {
            options = opts;
            options.insert(s!("include_docs"), s!("true"));
        } else {
            options = hashmap!{
                s!("include_docs") => s!("true")
            };
        }

        let response = self._client.get(
            self.create_document_path("_all_docs".into()),
            Some(options)
        )
        .send()
        .unwrap();

        Ok(DocumentCollection::new(from_reader(response).unwrap()))
    }

    /// Finds a document in the database through a Mango query. Parameters here http://docs.couchdb.org/en/latest/api/database/find.html
    pub fn find(&self, params: Value) -> Result<DocumentCollection, SofaError> {
        let path = self.create_document_path("_find".into());
        let response = self._client.post(path, js!(&params))
            .send()
            .unwrap();

        let data: FindResult = from_reader(response).unwrap();
        if let Some(doc_val) = data.docs {
            let documents: Vec<Document> = doc_val.iter()
                .filter(|d| { // Remove _design documents
                    let id: String = json_extr!(d["_id"]);
                    id.chars().nth(0).unwrap() != '_'
                })
                .map(|v| Document::new(v.clone()))
                .collect();

            Ok(DocumentCollection::new_from_documents(documents))
        } else if let Some(err) = data.error {
            Err(SofaError::from(err))
        } else {
            Ok(DocumentCollection::default())
        }
    }

    /// Updates a document
    pub fn save(&self, doc: Document) -> Result<Document, SofaError> {
        let id = doc._id.to_owned();
        let raw = doc.get_data();

        let response = self._client.put(
            self.create_document_path(id),
            to_string(&raw).unwrap()
        )
        .send()
        .unwrap();

        let data: DocumentCreatedResult = from_reader(response).unwrap();
        if !data.ok.is_some() || !data.ok.unwrap() {
            return Err(SofaError::from(data.error.unwrap()))
        }

        let mut val = doc.get_data();
        val["_rev"] = json!(data.rev.unwrap());

        Ok(Document::new(val))
    }

    /// Creates a document from a raw JSON document Value.
    pub fn create(&self, raw_doc: Value) -> Result<Document, SofaError> {
        let response = self._client.post(
            self.name.clone(),
            to_string(&raw_doc).unwrap()
        )
        .send()
        .unwrap();

        let data: DocumentCreatedResult = from_reader(response).unwrap();
        if !data.ok.is_some() || !data.ok.unwrap() {
            return Err(SofaError::from(data.error.unwrap()))
        }

        let mut val = raw_doc.clone();

        val["_id"] = json!(data.id.unwrap());
        val["_rev"] = json!(data.rev.unwrap());

        Ok(Document::new(val))
    }

    /// Removes a document from the database. Returns success in a `bool`
    pub fn remove(&self, doc: Document) -> bool {
        let response = self._client.delete(
            self.create_document_path(doc._id.clone()),
            Some(hashmap!{
                s!("rev") => doc._rev.clone()
            })
        )
        .send()
        .unwrap();

        match response.status() {
            StatusCode::Ok | StatusCode::Accepted => true,
            _ => false
        }
    }

    /// Inserts an index in a naive way, if it already exists, will throw an `Err`
    pub fn insert_index(&self, name: String, spec: IndexFields) -> Result<IndexCreated, SofaError> {
        let response = self._client.post(
            self.create_document_path("_index".into()),
            js!(json!({
                "name": name,
                "index": spec
            }))
        )
        .send()
        .unwrap();

        let data: IndexCreated = from_reader(response).unwrap();
        if data.error.is_some() {
            Err(SofaError::from(data.error.unwrap()))
        } else {
            Ok(data)
        }
    }

    /// Reads the database's indexes and returns them
    pub fn read_indexes(&self) -> DatabaseIndexList {
        let response = self._client.get(
            self.create_document_path("_index".into()),
            None
        )
        .send()
        .unwrap();

        from_reader(response).unwrap()
    }

    /// Method to ensure an index is created on the database with the following spec.
    /// Returns `true` when we created a new one, or `false` when the index was already existing.
    pub fn ensure_index(&self, name: String, spec: IndexFields) -> Result<bool, SofaError> {
        let db_indexes = self.read_indexes();

        // We look for our index
        for i in db_indexes.indexes.iter() {
            if i.name == name {
                // Found? Ok let's return
                return Ok(false);
            }
        }

        // Let's create it then
        let res = self.insert_index(name, spec);
        if res.is_err() {
            return Err(res.err().unwrap());
        }

        // Created and alright
        Ok(true)
    }
}
