use crate::client::Client;
use crate::document::{Document, DocumentCollection};
use crate::error::CouchError;
use crate::types::design::DesignCreated;
use crate::types::document::{DocumentCreatedResult, DocumentId};
use crate::types::find::{FindQuery, FindResult};
use crate::types::index::{DatabaseIndexList, IndexFields};
use crate::types::query::QueryParams;
use crate::types::view::ViewCollection;
use reqwest::{RequestBuilder, StatusCode};
use serde_json::{json, to_string, Value};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

/// Database holds the logic of making operations on a CouchDB Database
/// (sometimes called Collection in other NoSQL flavors such as MongoDB).
#[derive(Debug, Clone)]
pub struct Database {
    _client: Client,
    name: String,
}

impl Database {
    pub fn new(name: String, client: Client) -> Database {
        Database { _client: client, name }
    }

    // convenience function to retrieve the name of the database
    pub fn name(&self) -> &str {
        &self.name
    }

    fn create_document_path(&self, id: DocumentId) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/");
        result.push_str(&id);
        result
    }

    fn create_design_path(&self, id: DocumentId) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/_design/");
        result.push_str(&id);
        result
    }

    fn create_query_view_path(&self, design_id: DocumentId, view_id: DocumentId) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/_design/");
        result.push_str(&design_id);
        result.push_str("/_view/");
        result.push_str(&view_id);
        result
    }

    fn create_execute_update_path(
        &self,
        design_id: DocumentId,
        update_id: DocumentId,
        document_id: DocumentId,
    ) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/_design/");
        result.push_str(&design_id);
        result.push_str("/_update/");
        result.push_str(&update_id);
        result.push_str("/");
        result.push_str(&document_id);
        result
    }

    fn create_compact_path(&self, design_name: &str) -> String {
        let mut result: String = self.name.clone();
        result.push_str("/_compact/");
        result.push_str(design_name);
        result
    }

    async fn is_accepted(&self, request: Result<RequestBuilder, CouchError>) -> bool {
        if let Ok(req) = request {
            if let Ok(res) = req.send().await {
                return res.status() == StatusCode::ACCEPTED;
            }
        }

        false
    }

    async fn is_ok(&self, request: Result<RequestBuilder, CouchError>) -> bool {
        if let Ok(req) = request {
            if let Ok(res) = req.send().await {
                return match res.status() {
                    StatusCode::OK | StatusCode::NOT_MODIFIED => true,
                    _ => false,
                };
            }
        }

        false
    }

    /// Launches the compact process
    pub async fn compact(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_compact");

        let request = self._client.post(path, "".into());
        self.is_accepted(request).await
    }

    /// Starts the compaction of all views
    pub async fn compact_views(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_view_cleanup");

        let request = self._client.post(path, "".into());
        self.is_accepted(request).await
    }

    /// Starts the compaction of a given index
    pub async fn compact_index(&self, index: &str) -> bool {
        let request = self._client.post(self.create_compact_path(index), "".into());
        self.is_accepted(request).await
    }

    /// Checks if a document ID exists
    pub async fn exists(&self, id: DocumentId) -> bool {
        let request = self._client.head(self.create_document_path(id), None);
        self.is_ok(request).await
    }

    /// Gets one document
    pub async fn get(&self, id: DocumentId) -> Result<Document, CouchError> {
        let response = self
            ._client
            .get(self.create_document_path(id), None)?
            .send()
            .await?
            .error_for_status()?;
        Ok(Document::new(response.json().await?))
    }

    /// Gets documents in bulk with provided IDs list
    pub async fn get_bulk(&self, ids: Vec<DocumentId>) -> Result<DocumentCollection, CouchError> {
        self.get_bulk_params(ids, None).await
    }

    /// Each time a document is stored or updated in CouchDB, the internal B-tree is updated.
    /// Bulk insertion provides efficiency gains in both storage space, and time,
    /// by consolidating many of the updates to intermediate B-tree nodes.
    ///
    /// See the documentation on how to use bulk_docs here: https://docs.couchdb.org/en/stable/api/database/bulk-api.html#db-bulk-docs
    ///
    /// raw_docs is a vector of documents with or without an ID
    ///
    /// This endpoint can also be used to delete a set of documents by including "_deleted": true, in the document to be deleted.
    /// When deleting or updating, both _id and _rev are mandatory.
    pub async fn bulk_docs(&self, raw_docs: Vec<Value>) -> Result<Vec<DocumentCreatedResult>, CouchError> {
        let mut body = HashMap::new();
        body.insert(s!("docs"), raw_docs);

        let response = self
            ._client
            .post(self.create_document_path("_bulk_docs".into()), to_string(&body)?)?
            .send()
            .await?;

        let data: Vec<DocumentCreatedResult> = response.json().await?;

        Ok(data)
    }

    /// Gets documents in bulk with provided IDs list, with added params. Params description can be found here: Parameters description can be found here: http://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view
    pub async fn get_bulk_params(
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

        let response = self
            ._client
            .post(self.create_document_path("_all_docs".into()), to_string(&body)?)?
            .query(&options)
            .send()
            .await?
            .error_for_status()?;

        Ok(DocumentCollection::new(response.json().await?))
    }

    /// Gets all the documents in database
    pub async fn get_all(&self) -> Result<DocumentCollection, CouchError> {
        self.get_all_params(None).await
    }

    /// Gets all documents in the database, using bookmarks to iterate through all the documents.
    /// Results are returned through an mpcs channel for async processing. Use this for very large
    /// databases only. Batch size can be requested. A value of 0, means the default batch_size of
    /// 1000 is used. max_results of 0 means all documents will be returned. A given max_results is
    /// always rounded *up* to the nearest multiplication of batch_size.
    /// This operation is identical to find_batched(FindQuery::find_all(), tx, batch_size, max_results)
    pub async fn get_all_batched(
        &self,
        tx: Sender<DocumentCollection>,
        batch_size: u64,
        max_results: u64,
    ) -> Result<u64, CouchError> {
        let query = FindQuery::find_all();
        self.find_batched(query, tx, batch_size, max_results).await
    }

    /// Finds documents in the database, using bookmarks to iterate through all the documents.
    /// Results are returned through an mpcs channel for async processing. Use this for very large
    /// databases only. Batch size can be requested. A value of 0, means the default batch_size of
    /// 1000 is used. max_results of 0 means all documents will be returned. A given max_results is
    /// always rounded *up* to the nearest multiplication of batch_size.
    pub async fn find_batched(
        &self,
        mut query: FindQuery,
        mut tx: Sender<DocumentCollection>,
        batch_size: u64,
        max_results: u64,
    ) -> Result<u64, CouchError> {
        let mut bookmark = Option::None;
        let limit = if batch_size > 0 { batch_size } else { 1000 };

        let mut results: u64 = 0;
        query.limit = Option::Some(limit);

        let maybe_err = loop {
            let mut segment_query = query.clone();
            segment_query.bookmark = bookmark.clone();
            let all_docs = match self.find(&query).await {
                Ok(docs) => docs,
                Err(err) => break Some(err),
            };

            if all_docs.total_rows == 0 {
                // no more rows
                break None;
            }

            if all_docs.bookmark.is_some() && all_docs.bookmark != bookmark {
                bookmark.replace(all_docs.bookmark.clone().unwrap_or_default());
            } else {
                // no bookmark, break the query loop
                break None;
            }

            results += all_docs.total_rows as u64;

            tx.send(all_docs).await.unwrap();

            if max_results > 0 && results >= max_results {
                break None;
            }
        };

        if let Some(err) = maybe_err {
            Err(err)
        } else {
            Ok(results)
        }
    }

    /// Gets all the documents in database, with applied parameters. Parameters description can be found here: http://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view
    pub async fn get_all_params(&self, params: Option<QueryParams>) -> Result<DocumentCollection, CouchError> {
        let mut options;
        if let Some(opts) = params {
            options = opts;
        } else {
            options = QueryParams::default();
        }

        options.include_docs = Some(true);

        // we use POST here, because this allows for a larger set of keys to be provided, compared
        // to a GET call. It provides the same functionality
        let response = self
            ._client
            .post(self.create_document_path("_all_docs".into()), js!(&options))?
            .send()
            .await?
            .error_for_status()?;

        Ok(DocumentCollection::new(response.json().await?))
    }

    /// Finds a document in the database through a Mango query.
    pub async fn find(&self, query: &FindQuery) -> Result<DocumentCollection, CouchError> {
        let path = self.create_document_path("_find".into());
        let response = self._client.post(path, js!(query))?.send().await?;
        let status = response.status();
        let data: FindResult = response.json().await.unwrap();

        if let Some(doc_val) = data.docs {
            let documents: Vec<Document> = doc_val
                .into_iter()
                .filter(|d| {
                    // Remove _design documents
                    let id: String = json_extr!(d["_id"]);
                    !id.starts_with('_')
                })
                .map(Document::new)
                .collect();

            let mut bookmark = Option::None;
            let returned_bookmark = data.bookmark.unwrap_or_default();

            if returned_bookmark != "nil" && returned_bookmark != "" {
                // a valid bookmark has been returned
                bookmark.replace(returned_bookmark);
            }

            Ok(DocumentCollection::new_from_documents(documents, bookmark))
        } else if let Some(err) = data.error {
            Err(CouchError::new(err, status))
        } else {
            Ok(DocumentCollection::default())
        }
    }

    /// Updates a document
    pub async fn save(&self, doc: Document) -> Result<Document, CouchError> {
        let id = doc._id.to_owned();
        let raw = doc.get_data();

        let response = self
            ._client
            .put(self.create_document_path(id), to_string(&raw)?)?
            .send()
            .await?;

        let status = response.status();
        let data: DocumentCreatedResult = response.json().await?;

        match data.ok {
            Some(true) => {
                let mut val = doc.get_data();
                val["_rev"] = json!(data.rev);

                Ok(Document::new(val))
            }
            _ => {
                let err = data.error.unwrap_or_else(|| s!("unspecified error"));
                Err(CouchError::new(err, status))
            }
        }
    }

    /// Creates a document from a raw JSON document Value.
    pub async fn create(&self, raw_doc: Value) -> Result<Document, CouchError> {
        let response = self
            ._client
            .post(self.name.clone(), to_string(&raw_doc)?)?
            .send()
            .await?;

        let status = response.status();
        let data: DocumentCreatedResult = response.json().await?;

        match data.ok {
            Some(true) => {
                let data_id = match data.id {
                    Some(id) => id,
                    _ => return Err(CouchError::new(s!("invalid id"), status)),
                };

                let data_rev = match data.rev {
                    Some(rev) => rev,
                    _ => return Err(CouchError::new(s!("invalid rev"), status)),
                };

                let mut val = raw_doc.clone();
                val["_id"] = json!(data_id);
                val["_rev"] = json!(data_rev);

                Ok(Document::new(val))
            }
            _ => {
                let err = data.error.unwrap_or_else(|| s!("unspecified error"));
                Err(CouchError::new(err, status))
            }
        }
    }

    /// Creates a view document.
    pub async fn create_view(&self, design_name: String, doc: Value) -> Result<DesignCreated, CouchError> {
        let response = self
            ._client
            .put(self.create_design_path(design_name), to_string(&doc)?)?
            .send()
            .await?;

        let response_status = response.status();
        let result: DesignCreated = response.json().await?;

        if response_status.is_success() {
            Ok(result)
        } else {
            match result.error {
                Some(e) => Err(CouchError {
                    status: response_status,
                    message: e,
                }),
                None => Err(CouchError {
                    status: response_status,
                    message: s!("unspecified error"),
                }),
            }
        }
    }

    /// Executes a query against a view.
    pub async fn query(
        &self,
        design_name: String,
        view_name: String,
        options: Option<QueryParams>,
    ) -> Result<ViewCollection, CouchError> {
        let response = self
            ._client
            .post(self.create_query_view_path(design_name, view_name), js!(&options))?
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Convenience function to execute an update function whose name matches design name.
    pub async fn execute_update(
        &self,
        design_id: String,
        name: String,
        document_id: String,
        body: Option<Value>,
    ) -> Result<String, CouchError> {
        let body = match body {
            Some(v) => to_string(&v)?,
            None => "".to_string(),
        };

        let response = self
            ._client
            .put(self.create_execute_update_path(design_id, name, document_id), body)?
            .send()
            .await?
            .error_for_status()?;

        Ok(response.text().await?)
    }

    /// Removes a document from the database. Returns success in a `bool`
    pub async fn remove(&self, doc: Document) -> bool {
        let request = self._client.delete(
            self.create_document_path(doc._id.clone()),
            Some({
                let mut h = HashMap::new();
                h.insert(s!("rev"), doc._rev.clone());
                h
            }),
        );

        self.is_ok(request).await
    }

    /// Inserts an index in a naive way, if it already exists, will throw an
    /// `Err`
    pub async fn insert_index(&self, name: String, spec: IndexFields) -> Result<DesignCreated, CouchError> {
        let response = self
            ._client
            .post(
                self.create_document_path("_index".into()),
                js!(json!({
                    "name": name,
                    "index": spec
                })),
            )?
            .send()
            .await?;

        let status = response.status();
        let data: DesignCreated = response.json().await?;

        if data.error.is_some() {
            let err = data.error.unwrap_or_else(|| s!("unspecified error"));
            Err(CouchError::new(err, status))
        } else {
            Ok(data)
        }
    }

    /// Reads the database's indexes and returns them
    pub async fn read_indexes(&self) -> Result<DatabaseIndexList, CouchError> {
        let response = self
            ._client
            .get(self.create_document_path("_index".into()), None)?
            .send()
            .await?;

        Ok(response.json().await?)
    }

    /// Method to ensure an index is created on the database with the following
    /// spec. Returns `true` when we created a new one, or `false` when the
    /// index was already existing.
    pub async fn ensure_index(&self, name: String, spec: IndexFields) -> Result<bool, CouchError> {
        let db_indexes = self.read_indexes().await?;

        // We look for our index
        for i in db_indexes.indexes.into_iter() {
            if i.name == name {
                // Found? Ok let's return
                return Ok(false);
            }
        }

        // Let's create it then
        let result: DesignCreated = self.insert_index(name, spec).await?;
        match result.error {
            Some(e) => Err(CouchError {
                status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                message: e,
            }),
            // Created and alright
            None => Ok(true),
        }
    }
}
