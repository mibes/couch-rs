use url::Url;
use std::collections::HashMap;
use std::time::Duration;
use std::error::Error;
use serde_json::from_reader;

use reqwest;
pub use reqwest::{Method, RequestBuilder, Response, StatusCode};

use ::database::*;
use ::types::*;
use ::error::SofaError;

/// Client handles the URI manipulation logic and the HTTP calls to the CouchDB REST API.
/// It is also responsible for the creation/access/destruction of databases.
#[derive(Debug, Clone)]
pub struct Client {
    _client: reqwest::Client,
    dbs: Vec<&'static str>,
    _gzip: bool,
    _timeout: u8,
    pub uri: String,
    pub db_prefix: String
}

impl Client {
    pub fn new(uri: String) -> Client {
        let client = reqwest::Client::builder()
            .gzip(true)
            .timeout(Duration::new(4, 0))
            .build().unwrap();

        Client {
            _client: client,
            uri: uri,
            _gzip: true,
            _timeout: 4,
            dbs: Vec::new(),
            db_prefix: String::new()
        }
    }

    fn create_client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .gzip(self._gzip)
            .timeout(Duration::new(self._timeout as u64, 0))
            .build().unwrap()
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

    pub fn gzip(&mut self, enabled: bool) -> &Self {
        self._gzip = enabled;
        self._client = self.create_client();
        self
    }

    pub fn timeout(&mut self, to: u8) -> &Self {
        self._timeout = to;
        self._client = self.create_client();
        self
    }

    pub fn list_dbs(&self) -> reqwest::Result<Vec<String>> {
        self.get(String::from("/_all_dbs"), None)
            .send()
            .unwrap()
            .json::<Vec<String>>()
    }

    fn build_dbname(&self, dbname: &'static str) -> String {
        self.db_prefix.clone() + dbname
    }

    pub fn db(&self, dbname: &'static str) -> Result<Database, SofaError> {
        let name = self.build_dbname(dbname);

        let db = Database::new(name.clone(), self.clone());

        let path = self.create_path(name, None);
        let status_req = self._client.head(&path)
            .header(reqwest::header::ContentType::json())
            .send()
            .unwrap();

        match status_req.status() {
            StatusCode::Ok => {
                return Ok(db);
            },
            _ => {
                let response = self._client.put(&path)
                    .header(reqwest::header::ContentType::json())
                    .send()
                    .unwrap();

                let s: CouchResponse = from_reader(response).unwrap();

                if let Some(ok) = s.ok {
                    if ok {
                        return Ok(db);
                    } else {
                        return Err(SofaError::from(s.error.unwrap()));
                    }
                }

                Err(SofaError::from(s.error.unwrap()))
            }
        }
    }

    pub fn destroy_db(&self, dbname: &'static str) -> bool {
        let path = self.create_path(self.build_dbname(dbname), None);
        let response = self._client.delete(&path)
            .header(reqwest::header::ContentType::json())
            .send()
            .unwrap();

        let s: CouchResponse = from_reader(response).unwrap();

        if let Some(ok) = s.ok {
            return ok
        }

        false
    }

    pub fn check_status(&self) -> Option<Result<CouchStatus, SofaError>> {
        let response = self._client.get(&self.uri)
            .header(reqwest::header::ContentType::json())
            .send()
            .unwrap();

        match from_reader(response) {
            Ok(status) => return Some(Ok(status)),
            Err(e) => {
                let desc = s!(e.description());
                return Some(Err(SofaError::from(desc)))
            }
        }
    }

    fn create_path(&self,
        path: String,
        args: Option<HashMap<String, String>>
    ) -> String {
        let mut uri = Url::parse(&self.uri);
        uri = uri.unwrap().join(&path);

        let mut final_uri = uri.unwrap();

        if let Some(ref map) = args {
            let mut qp = final_uri.query_pairs_mut();
            for (k, v) in map {
                qp.append_pair(k, v);
            }
        }

        final_uri.into_string()
    }

    pub fn req(&self,
        method: Method,
        path: String,
        opts: Option<HashMap<String, String>>
    ) -> RequestBuilder {
        let uri = self.create_path(path, opts);
        let mut req = self._client.request(method, &uri);
        req.header(reqwest::header::Referer::new(uri.clone()));
        req.header(reqwest::header::ContentType::json());
        req
    }

    pub fn get(&self, path: String, args: Option<HashMap<String, String>>) -> RequestBuilder {
        self.req(Method::Get, path, args)
    }

    pub fn post(&self, path: String, body: String) -> RequestBuilder {
        let mut req = self.req(Method::Post, path, None);
        req.body(body);
        req
    }

    pub fn put(&self, path: String, body: String) -> RequestBuilder {
        let mut req = self.req(Method::Put, path, None);
        req.body(body);
        req
    }

    pub fn head(&self, path: String, args: Option<HashMap<String, String>>) -> RequestBuilder {
        self.req(Method::Head, path, args)
    }

    pub fn delete(&self, path: String, args: Option<HashMap<String, String>>) -> RequestBuilder {
        self.req(Method::Delete, path, args)
    }
}
