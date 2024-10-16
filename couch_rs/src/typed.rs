use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::changes::ChangesStream;
use crate::client::Client;
use crate::database::Database as RawDatabase;
use crate::document::{DocumentCollection, TypedCouchDocument};
use crate::error::CouchResult;
use crate::types::design::DesignCreated;
use crate::types::document::{DocumentCreatedResult, DocumentId};
use crate::types::find::FindQuery;
use crate::types::index::{DatabaseIndexList, IndexFields, IndexType};
use crate::types::query::{QueriesParams, QueryParams};
use crate::types::view::ViewCollection;

/// Wraps a database that will create/read/update/delete documents of a specific type.
/// This helps catching errors at compile time in case multiple instances of Database are used and each Database is supposed to handle a different type of document.
pub struct Database<T: TypedCouchDocument> {
    db: RawDatabase,
    phantom: PhantomData<T>,
}

impl<T: TypedCouchDocument> Database<T> {
    /// Creates a new typed Database instance.
    #[must_use]
    pub fn new(name: String, client: Client) -> Self {
        let db = RawDatabase::new(name, client);
        Self {
            db,
            phantom: PhantomData,
        }
    }

    // delegate all methods from RawDatabase

    /// See [`Database::name`](crate::database::Database::name)
    #[must_use]
    pub fn name(&self) -> &str {
        self.db.name()
    }

    /// See [`Database::compact`](crate::database::Database::compact)
    pub async fn compact(&self) -> bool {
        self.db.compact().await
    }

    /// See [`Database::compact_views`](crate::database::Database::compact_views)
    pub async fn compact_views(&self) -> bool {
        self.db.compact_views().await
    }

    /// See [`Database::compact_index`](crate::database::Database::compact_index)
    pub async fn compact_index(&self, index: &str) -> bool {
        self.db.compact_index(index).await
    }

    /// See [`Database::exists`](crate::database::Database::exists)
    pub async fn exists(&self, id: &str) -> bool {
        self.db.exists(id).await
    }

    /// See [`Database::get`](crate::database::Database::get)
    pub async fn get(&self, id: &str) -> CouchResult<T> {
        self.db.get(id).await
    }

    /// See [`Database::get_bulk`](crate::database::Database::get_bulk)
    pub async fn get_bulk(&self, ids: Vec<DocumentId>) -> CouchResult<DocumentCollection<T>> {
        self.db.get_bulk(ids).await
    }

    /// See [`Database::bulk_docs`](crate::database::Database::bulk_docs)
    pub async fn bulk_docs(&self, raw_docs: &mut [T]) -> CouchResult<Vec<DocumentCreatedResult>> {
        self.db.bulk_docs(raw_docs).await
    }

    /// See [`Database::get_bulk_params`](crate::database::Database::get_bulk_params)
    pub async fn get_bulk_params(
        &self,
        ids: Vec<DocumentId>,
        params: Option<QueryParams<DocumentId>>,
    ) -> CouchResult<DocumentCollection<T>> {
        self.db.get_bulk_params(ids, params).await
    }

    /// See [`Database::get_all`](crate::database::Database::get_all)
    pub async fn get_all(&self) -> CouchResult<DocumentCollection<T>> {
        self.db.get_all().await
    }

    /// See [`Database::get_all_batched`](crate::database::Database::get_all_batched)
    pub async fn get_all_batched(
        &self,
        tx: Sender<DocumentCollection<T>>,
        batch_size: u64,
        max_results: u64,
    ) -> CouchResult<u64> {
        self.db.get_all_batched(tx, batch_size, max_results).await
    }

    /// See [`Database::find_batched`](crate::database::Database::find_batched)
    pub async fn find_batched(
        &self,
        query: FindQuery,
        tx: Sender<DocumentCollection<T>>,
        batch_size: u64,
        max_results: u64,
    ) -> CouchResult<u64> {
        self.db.find_batched(query, tx, batch_size, max_results).await
    }

    /// See [`Database::query_many_all_docs`](crate::database::Database::query_many_all_docs)
    pub async fn query_many_all_docs(
        &self,
        queries: QueriesParams,
    ) -> CouchResult<Vec<ViewCollection<Value, Value, Value>>> {
        self.db.query_many_all_docs(queries).await
    }

    /// See [`Database::query_many`](crate::database::Database::query_many)
    pub async fn query_many(
        &self,
        design_name: &str,
        view_name: &str,
        queries: QueriesParams,
    ) -> CouchResult<Vec<ViewCollection<Value, Value, Value>>> {
        self.db.query_many(design_name, view_name, queries).await
    }

    /// See [`Database::get_all_params`](crate::database::Database::get_all_params)
    pub async fn get_all_params(&self, params: Option<QueryParams<DocumentId>>) -> CouchResult<DocumentCollection<T>> {
        self.db.get_all_params(params).await
    }

    /// See [`Database::find`](crate::database::Database::find)
    pub async fn find(&self, query: &FindQuery) -> CouchResult<DocumentCollection<T>> {
        self.db.find(query).await
    }

    /// See [`Database::save`](crate::database::Database::save)
    pub async fn save(&self, doc: &mut T) -> DocumentCreatedResult {
        self.db.save(doc).await
    }

    /// See [`Database::create`](crate::database::Database::create)
    pub async fn create(&self, doc: &mut T) -> DocumentCreatedResult {
        self.db.create(doc).await
    }

    /// See [`Database::upsert`](crate::database::Database::upsert)
    pub async fn upsert(&self, doc: &mut T) -> DocumentCreatedResult {
        self.db.upsert(doc).await
    }

    /// See [`Database::bulk_upsert`](crate::database::Database::bulk_upsert)
    pub async fn bulk_upsert(&self, docs: &mut [T]) -> CouchResult<Vec<DocumentCreatedResult>> {
        self.db.bulk_upsert(docs).await
    }

    /// See [`Database::create_view`](crate::database::Database::create_view)
    pub async fn create_view<V: Into<Value>>(&self, design_name: &str, views: V) -> CouchResult<DesignCreated> {
        self.db.create_view(design_name, views).await
    }

    /// See [`Database::query`](crate::database::Database::query)
    pub async fn query<K: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug + Clone, V: DeserializeOwned>(
        &self,
        design_name: &str,
        view_name: &str,
        options: Option<QueryParams<K>>,
    ) -> CouchResult<ViewCollection<K, V, T>> {
        self.db.query(design_name, view_name, options).await
    }

    /// See [`Database::execute_update`](crate::database::Database::execute_update)
    pub async fn execute_update(
        &self,
        design_id: &str,
        name: &str,
        document_id: &str,
        body: Option<Value>,
    ) -> CouchResult<String> {
        self.db.execute_update(design_id, name, document_id, body).await
    }

    /// See [`Database::remove`](crate::database::Database::remove)
    pub async fn remove(&self, doc: &T) -> bool {
        self.db.remove(doc).await
    }

    /// See [`Database::insert_index`](crate::database::Database::insert_index)
    pub async fn insert_index(
        &self,
        name: &str,
        def: IndexFields,
        index_type: Option<IndexType>,
        ddoc: Option<DocumentId>,
    ) -> CouchResult<DesignCreated> {
        self.db.insert_index(name, def, index_type, ddoc).await
    }

    /// See [`Database::read_indexes`](crate::database::Database::read_indexes)
    pub async fn read_indexes(&self) -> CouchResult<DatabaseIndexList> {
        self.db.read_indexes().await
    }

    /// See [`Database::delete_index`](crate::database::Database::delete_index)
    pub async fn delete_index(&self, ddoc: DocumentId, name: String) -> CouchResult<bool> {
        self.db.delete_index(ddoc, name).await
    }

    /// See [`Database::changes`](crate::database::Database::changes)
    #[must_use]
    pub fn changes(&self, last_seq: Option<Value>) -> ChangesStream {
        self.db.changes(last_seq)
    }
}
