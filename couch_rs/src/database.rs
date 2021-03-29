use crate::document::{DocumentCollection, TypedCouchDocument};
use crate::error::{CouchError, CouchResult};
use crate::types::design::DesignCreated;
use crate::types::document::{DocumentCreatedResponse, DocumentId};
use crate::types::find::{FindQuery, FindResult};
use crate::types::index::{DatabaseIndexList, IndexFields};
use crate::types::query::{QueriesCollection, QueriesParams, QueryParams};
use crate::types::view::ViewCollection;
use crate::{client::Client, types::document::DocumentCreatedResult};
use crate::{
    client::{is_accepted, is_ok},
    types::document::DocumentCreatedDetails,
};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::{json, to_string, Value};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

/// Database operations on a CouchDB Database
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

    fn create_raw_path(&self, id: &str) -> String {
        format!("{}/{}", self.name, id)
    }

    fn create_document_path(&self, id: &str) -> String {
        let encoded = url_encode!(id);
        format!("{}/{}", self.name, encoded)
    }

    fn create_design_path(&self, id: &str) -> String {
        let encoded = url_encode!(id);
        format!("{}/_design/{}", self.name, encoded)
    }

    fn create_query_view_path(&self, design_id: &str, view_id: &str) -> String {
        let encoded_design = url_encode!(design_id);
        let encoded_view = url_encode!(view_id);
        format!("{}/_design/{}/_view/{}", self.name, encoded_design, encoded_view)
    }

    fn create_execute_update_path(&self, design_id: &str, update_id: &str, document_id: &str) -> String {
        let encoded_design = url_encode!(design_id);
        let encoded_update = url_encode!(update_id);
        let encoded_document = url_encode!(document_id);
        format!(
            "{}/_design/{}/_update/{}/{}",
            self.name, encoded_design, encoded_update, encoded_document
        )
    }

    fn create_compact_path(&self, design_name: &str) -> String {
        let encoded_design = url_encode!(design_name);
        format!("{}/_compact/{}", self.name, encoded_design)
    }

    /// Launches the compact process
    pub async fn compact(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_compact");

        let request = self._client.post(path, "".into());
        is_accepted(request).await
    }

    /// Starts the compaction of all views
    pub async fn compact_views(&self) -> bool {
        let mut path: String = self.name.clone();
        path.push_str("/_view_cleanup");

        let request = self._client.post(path, "".into());
        is_accepted(request).await
    }

    /// Starts the compaction of a given index
    pub async fn compact_index(&self, index: &str) -> bool {
        let request = self._client.post(self.create_compact_path(index), "".into());
        is_accepted(request).await
    }

    /// Checks if a document ID exists
    ///
    /// Usage:
    /// ```
    /// use couch_rs::error::CouchResult;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     // check if the design document "_design/clip_view" exists
    ///     if db.exists("_design/clip_view").await {
    ///         println!("The design document exists");
    ///     }   
    ///
    ///     return Ok(());
    /// }
    /// ```
    pub async fn exists(&self, id: &str) -> bool {
        let request = self._client.head(self.create_document_path(id), None);
        is_ok(request).await
    }

    /// Convenience wrapper around get::<Value>(id)
    pub async fn get_raw(&self, id: &str) -> CouchResult<Value> {
        self.get(id).await
    }

    /// Gets one document
    ///
    /// Usage:
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use serde_json::{from_value, to_value, Value};
    /// use couch_rs::types::document::DocumentId;
    /// use couch_rs::document::TypedCouchDocument;
    /// use couch_rs::CouchDocument;
    /// use serde::{Deserialize, Serialize};
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[derive(Serialize, Deserialize, CouchDocument)]
    /// pub struct UserDetails {
    ///     #[serde(skip_serializing_if = "String::is_empty")]
    ///     pub _id: DocumentId,
    ///     #[serde(skip_serializing_if = "String::is_empty")]
    ///     pub _rev: String,
    ///     #[serde(rename = "firstName")]
    ///     pub first_name: Option<String>,
    ///     #[serde(rename = "lastName")]
    ///     pub last_name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     // before we can get the document, we need to create it first...
    ///     let seed_doc = UserDetails {
    ///         _id: "1234".to_string(),
    ///         _rev: "".to_string(),
    ///         first_name: None,
    ///         last_name: "Doe".to_string(),
    ///     };
    ///     let mut value = to_value(seed_doc)?;
    ///     db.create(&mut value).await?;
    ///
    ///     // now that the document is created, we can get it; typed:
    ///     let _user_details: UserDetails = db.get("1234").await?;
    ///
    ///     // now that the document is created, we can get it; or untyped:
    ///     let _raw_user: Value = db.get("1234").await?;
    ///
    ///     Ok(())
    /// }
    ///```
    pub async fn get<T: TypedCouchDocument>(&self, id: &str) -> CouchResult<T> {
        self._client
            .get(self.create_document_path(id), None)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(CouchError::from)
    }

    /// Gets documents in bulk with provided IDs list
    pub async fn get_bulk<T: TypedCouchDocument>(&self, ids: Vec<DocumentId>) -> CouchResult<DocumentCollection<T>> {
        self.get_bulk_params(ids, None).await
    }

    /// Gets documents in bulk with provided IDs list, as raw Values
    pub async fn get_bulk_raw(&self, ids: Vec<DocumentId>) -> CouchResult<DocumentCollection<Value>> {
        self.get_bulk_params(ids, None).await
    }

    /// Each time a document is stored or updated in CouchDB, the internal B-tree is updated.
    /// Bulk insertion provides efficiency gains in both storage space, and time,
    /// by consolidating many of the updates to intermediate B-tree nodes.
    ///
    /// See the documentation on how to use bulk_docs here: [db-bulk-docs](https://docs.couchdb.org/en/stable/api/database/bulk-api.html#db-bulk-docs)
    ///
    /// raw_docs is a vector of documents with or without an ID
    ///
    /// This endpoint can also be used to delete a set of documents by including "_deleted": true, in the document to be deleted.
    /// When deleting or updating, both _id and _rev are mandatory.
    ///
    /// Usage:
    /// ```
    /// use couch_rs::error::CouchResult;
    /// use serde_json::json;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///    let client = couch_rs::Client::new_local_test()?;
    ///    let db = client.db(TEST_DB).await?;
    ///
    ///    let _ndoc_result = db
    ///         .bulk_docs(&mut vec![
    ///             json!({"_id": "first", "thing": true}),
    ///             json!({"_id": "second", "thing": false}),
    ///         ]).await?;
    ///
    ///    return Ok(());
    /// }
    /// ```
    pub async fn bulk_docs<T: TypedCouchDocument>(
        &self,
        raw_docs: &mut [T],
    ) -> CouchResult<Vec<CouchResult<DocumentCreatedDetails>>> {
        let body = format!(r#"{{"docs":{} }}"#, to_string(raw_docs)?);
        dbg!(&body);
        let response = self
            ._client
            .post(self.create_raw_path("_bulk_docs"), body)
            .send()
            .await?;

        let data: Vec<DocumentCreatedResponse> = response.json().await?;
        dbg!(&data);

        if raw_docs.len() != data.len() {
            return Err(CouchError::new(
                format!(
                    "Unexpected size of response: {} given size of request: {}",
                    data.len(),
                    raw_docs.len()
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        let result = raw_docs
            .iter_mut()
            .zip(data.into_iter())
            .map(|(doc, response): (&mut T, DocumentCreatedResponse)| {
                let result: DocumentCreatedResult = response.into();
                match result {
                    Ok(r) => {
                        doc.set_id(r.id.as_str());
                        doc.set_rev(r.rev.as_str());
                        Ok(r)
                    }
                    Err(e) => Err(e),
                }
            })
            .collect();
        Ok(result)
    }

    /// Gets documents in bulk with provided IDs list, with added params. Params description can be found here:
    /// [_all_docs](https://docs.couchdb.org/en/latest/api/database/bulk-api.html?highlight=_all_docs)
    ///
    /// Usage:
    ///
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use serde_json::json;
    /// use serde_json::Value;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///     let mut doc_1 = json!({
    ///                     "_id": "john",
    ///                     "first_name": "John",
    ///                     "last_name": "Doe"
    ///                 });
    ///
    ///     let mut doc_2 = json!({
    ///                     "_id": "jane",
    ///                     "first_name": "Jane",
    ///                     "last_name": "Doe"
    ///                 });
    ///
    ///     // Save these documents
    ///     db.save(&mut doc_1).await?;
    ///     db.save(&mut doc_2).await?;
    ///
    ///     // subsequent call updates the existing document
    ///     let docs = db.get_bulk_params::<Value>(vec!["john".to_string(), "jane".to_string()], None).await?;
    ///
    ///     // verify that we received the 2 documents
    ///     assert_eq!(docs.rows.len(), 2);
    ///     Ok(())
    /// }
    /// ```   
    pub async fn get_bulk_params<T: TypedCouchDocument>(
        &self,
        ids: Vec<DocumentId>,
        params: Option<QueryParams>,
    ) -> CouchResult<DocumentCollection<T>> {
        let mut options = params.unwrap_or_default();

        options.include_docs = Some(true);
        options.keys = ids;

        let response = self
            ._client
            .post(self.create_raw_path("_all_docs"), to_string(&options)?)
            .send()
            .await?
            .error_for_status()?;

        Ok(DocumentCollection::new(response.json().await?))
    }

    /// Gets all the documents in database
    pub async fn get_all<T: TypedCouchDocument>(&self) -> CouchResult<DocumentCollection<T>> {
        self.get_all_params(None).await
    }

    /// Gets all the documents in database as raw Values
    pub async fn get_all_raw(&self) -> CouchResult<DocumentCollection<Value>> {
        self.get_all_params(None).await
    }

    /// Gets all documents in the database, using bookmarks to iterate through all the documents.
    /// Results are returned through an mpcs channel for async processing. Use this for very large
    /// databases only. Batch size can be requested. A value of 0, means the default batch_size of
    /// 1000 is used. max_results of 0 means all documents will be returned. A given max_results is
    /// always rounded *up* to the nearest multiplication of batch_size.
    /// This operation is identical to find_batched(FindQuery::find_all(), tx, batch_size, max_results)
    ///
    /// Check out the async_batch_read example for usage details
    pub async fn get_all_batched<T: TypedCouchDocument>(
        &self,
        tx: Sender<DocumentCollection<T>>,
        batch_size: u64,
        max_results: u64,
    ) -> CouchResult<u64> {
        let query = FindQuery::find_all();
        self.find_batched(query, tx, batch_size, max_results).await
    }

    /// Finds documents in the database, using bookmarks to iterate through all the documents.
    /// Results are returned through an mpcs channel for async processing. Use this for very large
    /// databases only. Batch size can be requested. A value of 0, means the default batch_size of
    /// 1000 is used. max_results of 0 means all documents will be returned. A given max_results is
    /// always rounded *up* to the nearest multiplication of batch_size.
    ///
    /// Check out the async_batch_read example for usage details
    pub async fn find_batched<T: TypedCouchDocument>(
        &self,
        mut query: FindQuery,
        tx: Sender<DocumentCollection<T>>,
        batch_size: u64,
        max_results: u64,
    ) -> CouchResult<u64> {
        let mut bookmark = Option::None;
        let limit = if batch_size > 0 { batch_size } else { 1000 };

        let mut results: u64 = 0;
        query.limit = Option::Some(limit);

        let maybe_err = loop {
            let mut segment_query = query.clone();
            segment_query.bookmark = bookmark.clone();
            let all_docs = match self.find(&segment_query).await {
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

            if let Err(_err) = tx.send(all_docs).await {
                break None;
            }

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

    /// Executes multiple specified built-in view queries of all documents in this database.
    /// This enables you to request multiple queries in a single request, in place of multiple POST /{db}/_all_docs requests.
    /// [More information](https://docs.couchdb.org/en/stable/api/database/bulk-api.html#sending-multiple-queries-to-a-database)
    /// Parameters description can be found [here](https://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view)
    ///
    /// Usage:
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::types::query::{QueryParams, QueriesParams};
    /// use couch_rs::error::CouchResult;
    /// use serde_json::{json, Value};
    ///
    /// const TEST_DB: &str = "vehicles";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     // imagine we have a database (e.g. vehicles) with multiple documents of different types; e.g. cars, planes and boats
    ///     // document IDs have been generated taking this into account, so cars have IDs starting with "car:",
    ///     // planes have IDs starting with "plane:", and boats have IDs starting with "boat:"
    ///     //
    ///     // let's query for all cars and all boats, sending just 1 request
    ///     let mut cars = QueryParams::default();
    ///     cars.start_key = Some("car".to_string());
    ///     cars.end_key = Some("car:\u{fff0}".to_string());
    ///
    ///     let mut boats = QueryParams::default();
    ///     boats.start_key = Some("boat".to_string());
    ///     boats.end_key = Some("boat:\u{fff0}".to_string());
    ///
    ///     let mut collections = db.query_many_all_docs(QueriesParams::new(vec![cars, boats])).await?;
    ///     println!("Succeeded querying for cars and boats");
    ///     let mut collections = collections.iter_mut();
    ///     let car_collection = collections.next().unwrap();
    ///     println!("Retrieved cars {:?}", car_collection);
    ///     let boat_collection = collections.next().unwrap();
    ///     println!("Retrieved boats {:?}", boat_collection);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn query_many_all_docs(
        &self,
        queries: QueriesParams,
    ) -> CouchResult<Vec<ViewCollection<Value, Value, Value>>> {
        self.query_view_many(self.create_raw_path("_all_docs/queries"), queries)
            .await
    }

    /// Executes multiple queries against a view.
    pub async fn query_many(
        &self,
        design_name: &str,
        view_name: &str,
        queries: QueriesParams,
    ) -> CouchResult<Vec<ViewCollection<Value, Value, Value>>> {
        self.query_view_many(self.create_query_view_path(design_name, view_name), queries)
            .await
    }

    async fn query_view_many(
        &self,
        view_path: String,
        queries: QueriesParams,
    ) -> CouchResult<Vec<ViewCollection<Value, Value, Value>>> {
        // we use POST here, because this allows for a larger set of keys to be provided, compared
        // to a GET call. It provides the same functionality
        let response = self
            ._client
            .post(view_path, js!(&queries))
            .send()
            .await?
            .error_for_status()?;

        let results: QueriesCollection<Value, Value, Value> = response.json().await?;
        Ok(results.results)
    }

    pub async fn get_all_params_raw(&self, params: Option<QueryParams>) -> CouchResult<DocumentCollection<Value>> {
        self.get_all_params(params).await
    }

    /// Gets all the documents in database, with applied parameters.
    /// Parameters description can be found here: [api-ddoc-view](https://docs.couchdb.org/en/latest/api/ddoc/views.html#api-ddoc-view)
    pub async fn get_all_params<T: TypedCouchDocument>(
        &self,
        params: Option<QueryParams>,
    ) -> CouchResult<DocumentCollection<T>> {
        let mut options = params.unwrap_or_default();

        options.include_docs = Some(true);

        // we use POST here, because this allows for a larger set of keys to be provided, compared
        // to a GET call. It provides the same functionality
        let response = self
            ._client
            .post(self.create_raw_path("_all_docs"), js!(&options))
            .send()
            .await?
            .error_for_status()?;

        Ok(DocumentCollection::new(response.json().await?))
    }

    /// Finds a document in the database through a Mango query as raw Values.
    /// Convenience function for find::<Value>(query)
    ///
    /// Usage:
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use serde_json::Value;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///     let find_all = FindQuery::find_all();
    ///     let docs = db.find_raw(&find_all).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn find_raw(&self, query: &FindQuery) -> CouchResult<DocumentCollection<Value>> {
        self.find(query).await
    }

    /// Finds a document in the database through a Mango query.
    ///
    /// Usage:
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use serde_json::Value;
    /// use couch_rs::document::TypedCouchDocument;
    /// use couch_rs::types::document::DocumentId;
    /// use couch_rs::CouchDocument;
    /// use couch_rs::document::DocumentCollection;
    /// use serde::{Deserialize, Serialize};
    ///
    /// const TEST_DB: &str = "user_db";
    ///
    /// #[derive(Serialize, Deserialize, CouchDocument, Default, Debug)]
    /// pub struct TestDoc {
    ///     #[serde(skip_serializing_if = "String::is_empty")]
    ///     pub _id: DocumentId,
    ///     #[serde(skip_serializing_if = "String::is_empty")]
    ///     pub _rev: String,
    ///     pub first_name: String,
    ///     pub last_name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///     let find_all = FindQuery::find_all();
    ///     let docs: DocumentCollection<TestDoc> = db.find(&find_all).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn find<T: TypedCouchDocument>(&self, query: &FindQuery) -> CouchResult<DocumentCollection<T>> {
        let path = self.create_raw_path("_find");
        let response = self._client.post(path, js!(query)).send().await?;
        let status = response.status();
        let data: FindResult<T> = response.json().await?;

        if let Some(doc_val) = data.docs {
            let documents: Vec<T> = doc_val
                .into_iter()
                .filter(|d| {
                    // Remove _design documents
                    let id: String = d.get_id().into_owned();
                    !id.starts_with('_')
                })
                .collect();

            let mut bookmark = Option::None;
            let returned_bookmark = data.bookmark.unwrap_or_default();

            if returned_bookmark != "nil" && !returned_bookmark.is_empty() {
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

    /// Saves a document to CouchDB. When the provided document includes both an `_id` and a `_rev`
    /// CouchDB will attempt to update the document. When only an `_id` is provided, the `save`
    /// method behaves like `create` and will attempt to create the document.
    ///
    /// Usage:
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use serde_json::{from_value, to_value};
    /// use couch_rs::types::document::DocumentId;
    /// use couch_rs::document::TypedCouchDocument;
    /// use couch_rs::CouchDocument;
    /// use serde::{Deserialize, Serialize};
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[derive(Serialize, Deserialize, CouchDocument)]
    /// pub struct UserDetails {
    ///     #[serde(skip_serializing_if = "String::is_empty")]
    ///     pub _id: DocumentId,
    ///     #[serde(skip_serializing_if = "String::is_empty")]
    ///     pub _rev: String,
    ///     #[serde(rename = "firstName")]
    ///     pub first_name: Option<String>,
    ///     #[serde(rename = "lastName")]
    ///     pub last_name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     // before we can get the document, we need to create it first...
    ///     let seed_doc = UserDetails {
    ///         _id: "123".to_string(),
    ///         _rev: "".to_string(),
    ///         first_name: None,
    ///         last_name: "Doe".to_string(),
    ///     };
    ///     let mut value = to_value(seed_doc)?;
    ///     db.create(&mut value).await?;
    ///
    ///     // now that the document is created, we can get it, update it, and save it...
    ///     let mut user_details: UserDetails = db.get("123").await?;
    ///     user_details.first_name = Some("John".to_string());
    ///
    ///     db.save(&mut user_details).await?;
    ///     Ok(())
    /// }
    ///```
    pub async fn save<T: TypedCouchDocument>(&self, doc: &mut T) -> DocumentCreatedResult {
        let id = doc.get_id().to_string();
        let body = to_string(&doc)?;
        let response = self._client.put(self.create_document_path(&id), body).send().await?;
        let status = response.status();
        let data: DocumentCreatedResponse = response.json().await?;

        match (data.ok, data.id, data.rev) {
            (Some(true), Some(id), Some(rev)) => {
                doc.set_id(&id);
                doc.set_rev(&rev);
                Ok(DocumentCreatedDetails { id, rev })
            }
            _ => {
                let err = data.error.unwrap_or_else(|| s!("unspecified error"));
                Err(CouchError::new(err, status))
            }
        }
    }

    /// Creates a document from a raw JSON document Value.
    /// Usage:
    ///
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use serde_json::json;
    /// use couch_rs::document::TypedCouchDocument;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    /// let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///     let mut doc = json!({
    ///                     "first_name": "John",
    ///                     "last_name": "Doe"
    ///                 });
    ///
    ///     let details = db.create(&mut doc).await?;
    ///
    ///     // verify that this is the 1st revision of the document
    ///     assert!(details.rev.starts_with('1'));
    ///     Ok(())
    /// }
    /// ```
    pub async fn create<T: TypedCouchDocument>(&self, doc: &mut T) -> CouchResult<DocumentCreatedDetails> {
        let response = self._client.post(self.name.clone(), to_string(&doc)?).send().await?;

        let status = response.status();
        let data: DocumentCreatedResponse = response.json().await?;

        match data.ok {
            Some(true) => {
                let id = data.id.ok_or_else(|| CouchError::new(s!("invalid id"), status))?;
                let rev = data.rev.ok_or_else(|| CouchError::new(s!("invalid rev"), status))?;

                doc.set_id(&id);
                doc.set_rev(&rev);
                Ok(DocumentCreatedDetails { id, rev })
            }
            _ => {
                let err = data.error.unwrap_or_else(|| s!("unspecified error"));
                Err(CouchError::new(err, status))
            }
        }
    }

    /// The upsert function combines a `get` with a `save` function. If the document with the
    /// provided `_id` can be found it will be merged with the provided Document's value, otherwise
    /// the document will be created.
    /// This operation always performs a `get`, so if you have a documents `_rev` using a `save` is
    /// quicker. Same is true when you know a document does *not* exist.
    ///
    /// Usage:
    ///
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use couch_rs::error::CouchResult;
    /// use couch_rs::document::TypedCouchDocument;
    /// use serde_json::json;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///     let mut doc = json!({
    ///                     "_id": "doe",
    ///                     "first_name": "John",
    ///                     "last_name": "Doe"
    ///                 });
    ///
    ///     // initial call creates the document
    ///     db.upsert(&mut doc).await?;
    ///
    ///     // subsequent call updates the existing document
    ///     let details = db.upsert(&mut doc).await?;
    ///
    ///     // verify that this is the 2nd revision of the document
    ///     assert!(details.rev.starts_with('2'));
    ///     Ok(())
    /// }
    /// ```
    pub async fn upsert<T: TypedCouchDocument>(&self, doc: &mut T) -> DocumentCreatedResult {
        let id = doc.get_id();

        match self.get::<T>(&id).await {
            Ok(current_doc) => {
                doc.set_rev(&current_doc.get_rev());
                self.save(doc).await
            }
            Err(err) => {
                if err.is_not_found() {
                    // document does not yet exist
                    self.save(doc).await
                } else {
                    Err(err)
                }
            }
        }
    }

    /// Creates a design with one of more view documents.
    ///
    /// Usage:
    /// ```
    /// use couch_rs::types::view::{CouchFunc, CouchViews};
    /// use couch_rs::error::CouchResult;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     let couch_func = CouchFunc {
    ///             map: "function (doc) { if (doc.funny == true) { emit(doc._id, doc.funny); } }".to_string(),
    ///             reduce: None,
    ///     };
    ///
    ///     let couch_views = CouchViews::new("clip_view", couch_func);
    ///     db.create_view("clip_design", couch_views).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_view<T: Into<serde_json::Value>>(
        &self,
        design_name: &str,
        views: T,
    ) -> CouchResult<DesignCreated> {
        let doc: Value = views.into();
        let response = self
            ._client
            .put(self.create_design_path(design_name), to_string(&doc)?)
            .send()
            .await?;

        let response_status = response.status();
        let result: DesignCreated = response.json().await?;

        if response_status.is_success() {
            Ok(result)
        } else {
            let error_msg = result.error.unwrap_or_else(|| s!("unspecified error"));
            Err(CouchError {
                id: result.id,
                status: response_status,
                message: error_msg,
            })
        }
    }

    /// Executes a query against a view, returning untyped Values
    pub async fn query_raw(
        &self,
        design_name: &str,
        view_name: &str,
        options: Option<QueryParams>,
    ) -> CouchResult<ViewCollection<Value, Value, Value>> {
        self.query(design_name, view_name, options).await
    }

    /// Executes a query against a view.
    ///
    /// Usage:
    /// ```
    /// use couch_rs::error::CouchResult;
    /// use couch_rs::types::view::RawViewCollection;
    /// use couch_rs::types::view::{CouchFunc, CouchViews};
    /// use serde_json::json;
    ///
    /// const TEST_DB: &str = "view_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    ///     let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     let mut doc = json!({
    ///                     "_id": "jdoe",
    ///                     "first_name": "John",
    ///                     "last_name": "Doe",
    ///                     "funny": true
    ///                 });
    ///
    ///     db.create(&mut doc).await?;
    ///
    ///     let couch_func = CouchFunc {
    ///             map: "function (doc) { if (doc.funny == true) { emit(doc._id, doc.funny); } }".to_string(),
    ///             reduce: None,
    ///     };
    ///
    ///     let couch_views = CouchViews::new("funny_guys", couch_func);
    ///     db.create_view("test_design", couch_views).await?;
    ///     let result: RawViewCollection<String, bool> = db.query("test_design", "funny_guys", None).await?;
    ///
    ///     println!("Funny guys:");
    ///     for item in result.rows.into_iter() {
    ///         let id = item.key;
    ///         let is_funny = item.value;
    ///         println!("{} is funny: {}", id, is_funny);
    ///     }
    ///     Ok(())
    /// }
    /// ```    
    pub async fn query<K: DeserializeOwned, V: DeserializeOwned, T: TypedCouchDocument>(
        &self,
        design_name: &str,
        view_name: &str,
        mut options: Option<QueryParams>,
    ) -> CouchResult<ViewCollection<K, V, T>> {
        if options.is_none() {
            options = Some(QueryParams::default());
        }

        self._client
            .post(self.create_query_view_path(design_name, view_name), js!(&options))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(CouchError::from)
    }

    /// Executes an update function.
    pub async fn execute_update(
        &self,
        design_id: &str,
        name: &str,
        document_id: &str,
        body: Option<Value>,
    ) -> CouchResult<String> {
        let body = match body {
            Some(v) => to_string(&v)?,
            None => String::default(),
        };

        self._client
            .put(self.create_execute_update_path(design_id, name, document_id), body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
            .map_err(CouchError::from)
    }

    /// Removes a document from the database. Returns success in a `bool`
    /// Usage:
    /// ```
    /// use couch_rs::types::find::FindQuery;
    /// use serde_json::{from_value, to_value, Value};
    /// use couch_rs::types::document::DocumentId;
    /// use couch_rs::error::CouchResult;
    ///
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> CouchResult<()> {
    /// let client = couch_rs::Client::new_local_test()?;
    ///     let db = client.db(TEST_DB).await?;
    ///
    ///     // first we need to get the document, because we need both the _id and _rev in order
    ///     // to delete
    ///     if let Some(doc) = db.get::<Value>("123").await.ok() {
    ///         db.remove(&doc).await;
    ///     }
    ///
    ///     Ok(())
    /// }
    ///```     
    pub async fn remove<T: TypedCouchDocument>(&self, doc: &T) -> bool {
        let request = self._client.delete(
            self.create_document_path(&doc.get_id()),
            Some({
                let mut h = HashMap::new();
                h.insert(s!("rev"), doc.get_rev().into_owned());
                h
            }),
        );

        is_ok(request).await
    }

    /// Inserts an index in a naive way, if it already exists, will throw an
    /// `Err`
    pub async fn insert_index(&self, name: &str, spec: IndexFields) -> CouchResult<DesignCreated> {
        let response = self
            ._client
            .post(
                self.create_raw_path("_index"),
                js!(json!({
                    "name": name,
                    "index": spec
                })),
            )
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
    pub async fn read_indexes(&self) -> CouchResult<DatabaseIndexList> {
        self._client
            .get(self.create_raw_path("_index"), None)
            .send()
            .await?
            .json()
            .await
            .map_err(CouchError::from)
    }

    /// Method to ensure an index is created on the database with the following
    /// spec. Returns `true` when we created a new one, or `false` when the
    /// index was already existing.
    pub async fn ensure_index(&self, name: &str, spec: IndexFields) -> CouchResult<bool> {
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
                id: result.id,
                status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                message: e,
            }),
            // Created and alright
            None => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_paths() {
        let client = Client::new_local_test().unwrap();
        let db = Database::new("testdb".to_string(), client);
        let p = db.create_raw_path("123");
        assert_eq!(p, "testdb/123");
        let p = db.create_document_path("1+3");
        assert_eq!(p, "testdb/1%2B3");
        let p = db.create_design_path("view1");
        assert_eq!(p, "testdb/_design/view1");
        let p = db.create_query_view_path("design1", "view1");
        assert_eq!(p, "testdb/_design/design1/_view/view1");
        let p = db.create_query_view_path("design+1", "view+1");
        assert_eq!(p, "testdb/_design/design%2B1/_view/view%2B1");
        let p = db.create_execute_update_path("design1", "update1", "123");
        assert_eq!(p, "testdb/_design/design1/_update/update1/123");
        let p = db.create_compact_path("view1");
        assert_eq!(p, "testdb/_compact/view1");
    }
}
