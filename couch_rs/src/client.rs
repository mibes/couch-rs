use crate::database::Database;
use crate::error::{CouchError, CouchResult};
use crate::types::system::{CouchResponse, CouchStatus, DbInfo};
use base64::write::EncoderWriter as Base64Encoder;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, REFERER, USER_AGENT};
use reqwest::{self, Method, StatusCode, Url};
use reqwest::{header, RequestBuilder};
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

fn construct_json_headers(uri: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if let Some(u) = uri {
        headers.insert(REFERER, HeaderValue::from_str(u).unwrap());
    }

    headers
}

fn parse_server(uri: &str) -> CouchResult<Url> {
    let parsed_url = Url::parse(uri)?;
    assert!(!parsed_url.cannot_be_a_base());
    Ok(parsed_url)
}

pub(crate) async fn is_accepted(request: RequestBuilder) -> bool {
    if let Ok(res) = request.send().await {
        res.status() == StatusCode::ACCEPTED
    } else {
        false
    }
}

pub(crate) async fn is_ok(request: RequestBuilder) -> bool {
    if let Ok(res) = request.send().await {
        matches!(res.status(), StatusCode::OK | StatusCode::NOT_MODIFIED)
    } else {
        false
    }
}

/// Client handles the URI manipulation logic and the HTTP calls to the CouchDB REST API.
/// It is also responsible for the creation/access/destruction of databases.
#[derive(Debug, Clone)]
pub struct Client {
    _client: reqwest::Client,
    dbs: Vec<&'static str>,
    _gzip: bool,
    _timeout: u64,
    uri: Url,
    username: Option<String>,
    password: Option<String>,
    pub db_prefix: String,
}

const TEST_DB_HOST: &str = "http://localhost:5984";
const TEST_DB_USER: &str = "admin";
const TEST_DB_PW: &str = "password";
const DEFAULT_TIME_OUT: u64 = 10;

impl Client {
    /// new creates a new Couch client with a default timeout of 10 seconds.
    /// The timeout is applied from when the request starts connecting until the response body has finished.
    /// The URI has to be in this format: http://hostname:5984, for example: http://192.168.64.5:5984
    pub fn new(uri: &str, username: &str, password: &str) -> CouchResult<Client> {
        Client::new_with_timeout(uri, Some(username), Some(password), DEFAULT_TIME_OUT)
    }

    /// new_no_auth creates a new Couch client with a default timeout of 10 seconds. *Without authentication*.
    /// The timeout is applied from when the request starts connecting until the response body has finished.
    /// The URI has to be in this format: http://hostname:5984, for example: http://192.168.64.5:5984
    pub fn new_no_auth(uri: &str) -> CouchResult<Client> {
        Client::new_with_timeout(uri, None, None, DEFAULT_TIME_OUT)
    }

    /// new_local_test creates a new Couch client *for testing purposes* with a default timeout of 10 seconds.
    /// The timeout is applied from when the request starts connecting until the response body has finished.
    /// The URI that will be used is: http://hostname:5984, with a username of "admin" and a password
    /// of "password". Use this only for testing!!!
    pub fn new_local_test() -> CouchResult<Client> {
        Client::new_with_timeout(TEST_DB_HOST, Some(TEST_DB_USER), Some(TEST_DB_PW), DEFAULT_TIME_OUT)
    }

    /// new_with_timeout creates a new Couch client. The URI has to be in this format: http://hostname:5984,
    /// The timeout is applied from when the request starts connecting until the response body has finished.
    /// Timeout is in seconds.
    pub fn new_with_timeout(
        uri: &str,
        username: Option<&str>,
        password: Option<&str>,
        timeout: u64,
    ) -> CouchResult<Client> {
        let mut headers = header::HeaderMap::new();

        if let Some(username) = username {
            let mut header_value = b"Basic ".to_vec();
            {
                let mut encoder = Base64Encoder::new(&mut header_value, base64::STANDARD);
                // The unwraps here are fine because Vec::write* is infallible.
                write!(encoder, "{}:", username).unwrap();
                if let Some(password) = password {
                    write!(encoder, "{}", password).unwrap();
                }
            }

            let auth_header = header::HeaderValue::from_bytes(&header_value).expect("can not set AUTHORIZATION header");
            headers.insert(header::AUTHORIZATION, auth_header);
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .gzip(true)
            .timeout(Duration::new(timeout, 0))
            .build()?;

        Ok(Client {
            _client: client,
            uri: parse_server(uri)?,
            _gzip: true,
            _timeout: timeout,
            dbs: Vec::new(),
            db_prefix: String::new(),
            username: username.map(|u| u.to_string()),
            password: password.map(|p| p.to_string()),
        })
    }

    pub fn get_self(&mut self) -> &mut Self {
        self
    }

    pub fn set_uri(&mut self, uri: &str) -> CouchResult<&Self> {
        self.uri = parse_server(uri)?;
        Ok(self)
    }

    pub fn set_prefix(&mut self, prefix: String) -> &Self {
        self.db_prefix = prefix;
        self
    }

    /// List the databases in CouchDB
    ///
    /// Usage:
    /// ```
    /// use std::error::Error;
    ///
    /// const DB_HOST: &str = "http://localhost:5984";
    /// const DB_USER: &str = "admin";
    /// const DB_PW: &str = "password";
    /// const TEST_DB: &str = "test_db";
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn Error>> {
    ///     let client = couch_rs::Client::new(DB_HOST, DB_USER, DB_PW)?;
    ///     let db = client.db(TEST_DB).await?;
    ///     let dbs = client.list_dbs().await?;
    ///     dbs.iter().for_each(|db| println!("Database: {}", db));
    ///     Ok(())
    /// }
    ///```     
    pub async fn list_dbs(&self) -> CouchResult<Vec<String>> {
        let response = self.get(String::from("/_all_dbs"), None).send().await?;
        let data = response.json().await?;

        Ok(data)
    }

    fn build_dbname(&self, dbname: &str) -> String {
        self.db_prefix.clone() + dbname
    }

    /// Connect to an existing database, or create a new one, when this one does not exist.
    pub async fn db(&self, dbname: &str) -> CouchResult<Database> {
        let name = self.build_dbname(dbname);

        let db = Database::new(name.clone(), self.clone());

        let head_response = self
            .head(name, None)
            .headers(construct_json_headers(None))
            .send()
            .await?;

        match head_response.status() {
            StatusCode::OK => Ok(db),
            _ => self.make_db(dbname).await,
        }
    }

    /// Create a new database with the given name
    pub async fn make_db(&self, dbname: &str) -> CouchResult<Database> {
        let name = self.build_dbname(dbname);

        let db = Database::new(name.clone(), self.clone());

        let put_response = self
            .put(name, String::default())
            .headers(construct_json_headers(None))
            .send()
            .await?;

        let status = put_response.status();
        let s: CouchResponse = put_response.json().await?;

        match s.ok {
            Some(true) => Ok(db),
            _ => {
                let err = s.error.unwrap_or_else(|| s!("unspecified error"));
                Err(CouchError::new(err, status))
            }
        }
    }

    /// Destroy the database with the given name
    pub async fn destroy_db(&self, dbname: &str) -> CouchResult<bool> {
        let response = self
            .delete(self.build_dbname(dbname), None)
            .headers(construct_json_headers(None))
            .send()
            .await?;

        let s: CouchResponse = response.json().await?;

        Ok(s.ok.unwrap_or(false))
    }

    /// Checks if a database exists
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
    ///     if client.exists(TEST_DB).await? {
    ///         println!("The database exists");
    ///     }
    ///
    ///     return Ok(());
    /// }
    /// ```
    pub async fn exists(&self, dbname: &str) -> CouchResult<bool> {
        let result = self.head(self.build_dbname(dbname), None).send().await?;
        Ok(result.status().is_success())
    }

    /// Gets information about the specified database.
    /// See [common](https://docs.couchdb.org/en/stable/api/database/common.html) for more details.
    pub async fn get_info(&self, dbname: &str) -> CouchResult<DbInfo> {
        let response = self
            .get(self.build_dbname(dbname), None)
            .send()
            .await?
            .error_for_status()?;
        let info = response.json().await?;
        Ok(info)
    }

    /// Returns meta information about the instance. The response contains information about the server,
    /// including a welcome message and the version of the server.
    /// See [common](https://docs.couchdb.org/en/stable/api/server/common.html) for more details.
    pub async fn check_status(&self) -> CouchResult<CouchStatus> {
        let response = self
            .get(String::default(), None)
            .headers(construct_json_headers(None))
            .send()
            .await?;

        let status = response.json().await?;
        Ok(status)
    }

    pub fn req(&self, method: Method, path: String, opts: Option<HashMap<String, String>>) -> RequestBuilder {
        let mut uri = self.uri.clone();
        uri.set_path(&path);

        if let Some(ref map) = opts {
            let mut qp = uri.query_pairs_mut();
            for (k, v) in map {
                qp.append_pair(k, v);
            }
        }

        self._client
            .request(method, uri.as_str())
            .headers(construct_json_headers(Some(uri.as_str())))
    }

    pub(crate) fn get(&self, path: String, args: Option<HashMap<String, String>>) -> RequestBuilder {
        self.req(Method::GET, path, args)
    }

    pub(crate) fn post(&self, path: String, body: String) -> RequestBuilder {
        self.req(Method::POST, path, None).body(body)
    }

    pub(crate) fn put(&self, path: String, body: String) -> RequestBuilder {
        self.req(Method::PUT, path, None).body(body)
    }

    pub(crate) fn head(&self, path: String, args: Option<HashMap<String, String>>) -> RequestBuilder {
        self.req(Method::HEAD, path, args)
    }

    pub(crate) fn delete(&self, path: String, args: Option<HashMap<String, String>>) -> RequestBuilder {
        self.req(Method::DELETE, path, args)
    }
}
