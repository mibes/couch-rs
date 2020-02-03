use std::collections::HashMap;

use reqwest::StatusCode;

use serde_json;
use serde_json::{from_reader, to_string, Value, json};
use crate::document::{Document, DocumentCollection};
use crate::error::CouchError;
use crate::client::Client;
use crate::types::document::{DocumentId, DocumentCreatedResult};
use crate::types::find::FindResult;
use crate::types::index::{IndexFields, IndexCreated, DatabaseIndexList};

/// Database holds the logic of making operations on a CouchDB Database
/// (sometimes called Collection in other NoSQL flavors such as MongoDB).
#[derive(Debug, Clone)]
pub struct Database {
    _client: Client,
    name: String,
}

impl Database {
    pub fn new(name: String, client: Client) -> Database {
        Database {
            _client: client,
            name: name,
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

        let request = self._client.post(path, "".into());

        request
            .and_then(|req| {
                Ok(req.send()
                    .and_then(|res| Ok(res.status() == StatusCode::ACCEPTED))
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    /// Starts the compaction of all views
    pub fn compact_views(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_view_cleanup");

        let request = self._client.post(path, "".into());

        request
            .and_then(|req| {
                Ok(req.send()
                    .and_then(|res| Ok(res.status() == StatusCode::ACCEPTED))
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    /// Starts the compaction of a given index
    pub fn compact_index(&self, index: &'static str) -> bool {
        let request = self._client.post(self.create_compact_path(index), "".into());

        request
            .and_then(|req| {
                Ok(req.send()
                    .and_then(|res| Ok(res.status() == StatusCode::ACCEPTED))
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    /// Checks if a document ID exists
    pub fn exists(&self, id: DocumentId) -> bool {
        let request = self._client.head(self.create_document_path(id), None);

        request
            .and_then(|req| {
                Ok(req.send()
                    .and_then(|res| {
                        Ok(match res.status() {
                            StatusCode::OK | StatusCode::NOT_MODIFIED => true,
                            _ => false,
                        })
                    })
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    /// Gets one document
    pub fn get(&self, id: DocumentId) -> Result<Document, CouchError> {
        let response = self._client.get(self.create_document_path(id), None)?.send()?;

        Ok(Document::new(from_reader(response)?))
    }

    /// Gets documents in bulk with provided IDs list
    pub fn get_bulk(&self, ids: Vec<DocumentId>) -> Result<DocumentCollection, CouchError> {
        self.get_bulk_params(ids, None)
    }

    /// Gets documents in bulk with provided IDs list, with added params. Params description can be found here: Parameters description can be found here: http://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view
    pub fn get_bulk_params(
        &self,
        ids: Vec<DocumentId>,
        params: Option<HashMap<String, String>>,
    ) -> Result<DocumentCollection, CouchError> {
        let mut options;
        if let Some(opts) = params {
            options = opts;
        } else {
            options = HashMap::new();
        }

        options.insert(s!("include_docs"), s!("true"));

        let mut body = HashMap::new();
        body.insert(s!("keys"), ids);

        let response = self._client
            .get(self.create_document_path("_all_docs".into()), Some(options))?
            .body(to_string(&body)?)
            .send()?;

        Ok(DocumentCollection::new(from_reader(response)?))
    }

    /// Gets all the documents in database
    pub fn get_all(&self) -> Result<DocumentCollection, CouchError> {
        self.get_all_params(None)
    }

    /// Gets all the documents in database, with applied parameters. Parameters description can be found here: http://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view
    pub fn get_all_params(&self, params: Option<HashMap<String, String>>) -> Result<DocumentCollection, CouchError> {
        let mut options;
        if let Some(opts) = params {
            options = opts;
        } else {
            options = HashMap::new();
        }

        options.insert(s!("include_docs"), s!("true"));

        let response = self._client
            .get(self.create_document_path("_all_docs".into()), Some(options))?
            .send()?;

        Ok(DocumentCollection::new(from_reader(response)?))
    }

    /// Finds a document in the database through a Mango query. Parameters here http://docs.couchdb.org/en/latest/api/database/find.html
    pub fn find(&self, params: Value) -> Result<DocumentCollection, CouchError> {
        let path = self.create_document_path("_find".into());
        let response = self._client.post(path, js!(&params))?.send()?;
        let status = response.status();

        let data: FindResult = from_reader(response)?;
        if let Some(doc_val) = data.docs {
            let documents: Vec<Document> = doc_val
                .into_iter()
                .filter(|d| {
                    // Remove _design documents
                    let id: String = json_extr!(d["_id"]);
                    !id.starts_with('_')
                })
                .map(|v| Document::new(v.clone()))
                .collect();

            Ok(DocumentCollection::new_from_documents(documents))
        } else if let Some(err) = data.error {
            Err(CouchError::new(err, status).into())
        } else {
            Ok(DocumentCollection::default())
        }
    }

    /// Updates a document
    pub fn save(&self, doc: Document) -> Result<Document, CouchError> {
        let id = doc._id.to_owned();
        let raw = doc.get_data();

        let response = self._client
            .put(self.create_document_path(id), to_string(&raw)?)?
            .send()?;

        let status = response.status();
        let data: DocumentCreatedResult = from_reader(response)?;

        match data.ok {
            Some(true) => {
                let mut val = doc.get_data();
                val["_rev"] = json!(data.rev);

                Ok(Document::new(val))
            }
            Some(false) | _ => {
                let err = data.error.unwrap_or(s!("unspecified error"));
                return Err(CouchError::new(err, status).into());
            }
        }
    }

    /// Creates a document from a raw JSON document Value.
    pub fn create(&self, raw_doc: Value) -> Result<Document, CouchError> {
        let response = self._client.post(self.name.clone(), to_string(&raw_doc)?)?.send()?;
        let status = response.status();

        let data: DocumentCreatedResult = from_reader(response)?;

        match data.ok {
            Some(true) => {
                let data_id = match data.id {
                    Some(id) => id,
                    _ => return Err(CouchError::new(s!("invalid id"), status).into()),
                };

                let data_rev = match data.rev {
                    Some(rev) => rev,
                    _ => return Err(CouchError::new(s!("invalid rev"), status).into()),
                };

                let mut val = raw_doc.clone();
                val["_id"] = json!(data_id);
                val["_rev"] = json!(data_rev);

                Ok(Document::new(val))
            }
            Some(false) | _ => {
                let err = data.error.unwrap_or(s!("unspecified error"));
                return Err(CouchError::new(err, status).into());
            }
        }
    }

    /// Removes a document from the database. Returns success in a `bool`
    pub fn remove(&self, doc: Document) -> bool {
        let request = self._client.delete(
            self.create_document_path(doc._id.clone()),
            Some({
                let mut h = HashMap::new();
                h.insert(s!("rev"), doc._rev.clone());
                h
            }),
        );

        request
            .and_then(|req| {
                Ok(req.send()
                    .and_then(|res| {
                        Ok(match res.status() {
                            StatusCode::OK | StatusCode::ACCEPTED => true,
                            _ => false,
                        })
                    })
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    /// Inserts an index in a naive way, if it already exists, will throw an
    /// `Err`
    pub fn insert_index(&self, name: String, spec: IndexFields) -> Result<IndexCreated, CouchError> {
        let response = self._client
            .post(
                self.create_document_path("_index".into()),
                js!(json!({
                "name": name,
                "index": spec
            })),
            )?
            .send()?;

        let status = response.status();
        let data: IndexCreated = from_reader(response)?;

        if data.error.is_some() {
            let err = data.error.unwrap_or(s!("unspecified error"));
            Err(CouchError::new(err, status).into())
        } else {
            Ok(data)
        }
    }

    /// Reads the database's indexes and returns them
    pub fn read_indexes(&self) -> Result<DatabaseIndexList, CouchError> {
        let response = self._client
            .get(self.create_document_path("_index".into()), None)?
            .send()?;

        Ok(from_reader(response)?)
    }

    /// Method to ensure an index is created on the database with the following
    /// spec. Returns `true` when we created a new one, or `false` when the
    /// index was already existing.
    pub fn ensure_index(&self, name: String, spec: IndexFields) -> Result<bool, CouchError> {
        let db_indexes = self.read_indexes()?;

        // We look for our index
        for i in db_indexes.indexes.into_iter() {
            if i.name == name {
                // Found? Ok let's return
                return Ok(false);
            }
        }

        // Let's create it then
        let _ = self.insert_index(name, spec)?;

        // Created and alright
        Ok(true)
    }
}
