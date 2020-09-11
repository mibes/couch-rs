use std::collections::HashMap;
use std::time::Duration;
use reqwest::RequestBuilder;
use reqwest::{self, Url, Method, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE, REFERER};
use crate::database::Database;
use crate::error::CouchError;
use crate::types::system::{CouchResponse, CouchStatus};

fn construct_json_headers(uri: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if let Some(u) = uri {
        headers.insert(REFERER, HeaderValue::from_str(u).unwrap());
    }

    headers
}

/// Client handles the URI manipulation logic and the HTTP calls to the CouchDB REST API.
/// It is also responsible for the creation/access/destruction of databases.
#[derive(Debug, Clone)]
pub struct Client {
    _client: reqwest::Client,
    dbs: Vec<&'static str>,
    _gzip: bool,
    _timeout: u64,
    pub uri: String,
    pub db_prefix: String,
}

impl Client {
    /// new creates a new Couch client with a default timeout of 5 seconds.
    /// The URI has to be in this format: http://hostname:5984, for example: http://192.168.64.5:5984
    pub fn new(uri: &str) -> Result<Client, CouchError> {
        Client::new_with_timeout(uri, 10)
    }

    /// new_with_timeout creates a new Couch client. The URI has to be in this format: http://hostname:5984,
    /// timeout is in seconds.
    pub fn new_with_timeout(uri: &str, timeout: u64) -> Result<Client, CouchError> {
        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::new(timeout, 0))
            .build()?;

        Ok(Client {
            _client: client,
            uri: uri.to_string(),
            _gzip: true,
            _timeout: timeout,
            dbs: Vec::new(),
            db_prefix: String::new(),
        })
    }

    fn create_client(&self) -> Result<reqwest::Client, CouchError> {
        let client = reqwest::Client::builder()
            .gzip(self._gzip)
            .timeout(Duration::new(self._timeout, 0))
            .build()?;

        Ok(client)
    }

    pub fn get_self(&mut self) -> &mut Self {
        self
    }

    pub fn set_uri(&mut self, uri: String) -> &Self {
        self.uri = uri;
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &Self {
        self.db_prefix = prefix;
        self
    }

    pub fn gzip(&mut self, enabled: bool) -> Result<&Self, CouchError> {
        self._gzip = enabled;
        self._client = self.create_client()?;

        Ok(self)
    }

    pub fn timeout(&mut self, to: u64) -> Result<&Self, CouchError> {
        self._timeout = to;
        self._client = self.create_client()?;

        Ok(self)
    }

    pub async fn list_dbs(&self) -> Result<Vec<String>, CouchError> {
        let response = self.get(String::from("/_all_dbs"), None)?.send().await?;
        let data = response.json().await?;

        Ok(data)
    }

    fn build_dbname(&self, dbname: &str) -> String {
        self.db_prefix.clone() + dbname
    }

    pub async fn db(&self, dbname: &str) -> Result<Database, CouchError> {
        let name = self.build_dbname(dbname);

        let db = Database::new(name.clone(), self.clone());

        let path = self.create_path(name, None)?;

        let head_response = self._client.head(&path)
            .headers(construct_json_headers(None))
            .send().await?;

        match head_response.status() {
            StatusCode::OK => Ok(db),
            _ => self.make_db(dbname).await,
        }
    }

    pub async fn make_db(&self, dbname: &str) -> Result<Database, CouchError> {
        let name = self.build_dbname(dbname);

        let db = Database::new(name.clone(), self.clone());

        let path = self.create_path(name, None)?;

        let put_response = self._client.put(&path)
            .headers(construct_json_headers(None))
            .send().await?;

        let status = put_response.status();
        let s: CouchResponse = put_response.json().await?;

        match s.ok {
            Some(true) => Ok(db),
            _ => {
                let err = s.error.unwrap_or_else(|| s!("unspecified error"));
                Err(CouchError::new(err, status))
            },
        }
    }

    pub async fn destroy_db(&self, dbname: &str) -> Result<bool, CouchError> {
        let path = self.create_path(self.build_dbname(dbname), None)?;
        let response = self._client.delete(&path)
            .headers(construct_json_headers(None))
            .send().await?;

        let s: CouchResponse = response.json().await?;

        Ok(s.ok.unwrap_or(false))
    }

    pub async fn check_status(&self) -> Result<CouchStatus, CouchError> {
        let response = self._client.get(&self.uri)
            .headers(construct_json_headers(None))
            .send().await?;

        let status = response.json().await?;

        Ok(status)
    }

    fn create_path(&self,
        path: String,
        args: Option<HashMap<String, String>>
    ) -> Result<String, CouchError> {
        let mut uri = Url::parse(&self.uri)?.join(&path)?;

        if let Some(ref map) = args {
            let mut qp = uri.query_pairs_mut();
            for (k, v) in map {
                qp.append_pair(k, v);
            }
        }

        Ok(uri.into_string())
    }

    pub fn req(&self,
        method: Method,
        path: String,
        opts: Option<HashMap<String, String>>
    ) -> Result<RequestBuilder, CouchError> {
        let uri = self.create_path(path, opts)?;
        let req = self._client.request(method, &uri).
            headers(construct_json_headers(Some(&uri)));

        // req.header(reqwest::header::Referer::new(uri.clone()));

        Ok(req)
    }

    pub fn get(&self, path: String, args: Option<HashMap<String, String>>) -> Result<RequestBuilder, CouchError> {
        Ok(self.req(Method::GET, path, args)?)
    }

    pub fn post(&self, path: String, body: String) -> Result<RequestBuilder, CouchError> {
        let req = self.req(Method::POST, path, None)?.body(body);
        Ok(req)
    }

    pub fn put(&self, path: String, body: String) -> Result<RequestBuilder, CouchError> {
        let req = self.req(Method::PUT, path, None)?.body(body);
        Ok(req)
    }

    pub fn head(&self, path: String, args: Option<HashMap<String, String>>) -> Result<RequestBuilder, CouchError> {
        Ok(self.req(Method::HEAD, path, args)?)
    }

    pub fn delete(&self, path: String, args: Option<HashMap<String, String>>) -> Result<RequestBuilder, CouchError> {
        Ok(self.req(Method::DELETE, path, args)?)
    }
}
