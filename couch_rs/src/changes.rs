use crate::client::Client;
use futures_core::{Future, Stream};
use futures_util::{ready, FutureExt, StreamExt, TryStreamExt};
use reqwest::StatusCode;
use reqwest::{Method, Response};
use std::collections::HashMap;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;
use tokio_util::io::StreamReader;

use crate::error::{CouchError, CouchResult};
use crate::types::changes::{ChangeEvent, Event};

/// The max timeout value for longpoll/continous HTTP requests
/// that CouchDB supports (see [1]).
///
/// [1]: https://docs.couchdb.org/en/stable/api/database/changes.html
const COUCH_MAX_TIMEOUT: usize = 60000;

/// The stream for the `_changes` endpoint.
///
/// This is returned from [Database::changes].
pub struct ChangesStream {
    last_seq: Option<serde_json::Value>,
    client: Client,
    database: String,
    state: ChangesStreamState,
    params: HashMap<String, String>,
    infinite: bool,
}

enum ChangesStreamState {
    Idle,
    Requesting(Pin<Box<dyn Future<Output = CouchResult<Response>>>>),
    Reading(Pin<Box<dyn Stream<Item = io::Result<String>>>>),
}

impl ChangesStream {
    /// Create a new changes stream.
    pub fn new(client: Client, database: String, last_seq: Option<serde_json::Value>) -> Self {
        let mut params = HashMap::new();
        params.insert("feed".to_string(), "continuous".to_string());
        params.insert("timeout".to_string(), "0".to_string());
        params.insert("include_docs".to_string(), "true".to_string());
        Self::with_params(client, database, last_seq, params)
    }

    /// Create a new changes stream with params.
    pub fn with_params(
        client: Client,
        database: String,
        last_seq: Option<serde_json::Value>,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            client,
            database,
            params,
            state: ChangesStreamState::Idle,
            infinite: false,
            last_seq,
        }
    }

    /// Set the starting seq.
    pub fn set_last_seq(&mut self, last_seq: Option<serde_json::Value>) {
        self.last_seq = last_seq;
    }

    /// Set infinite mode.
    ///
    /// If set to true, the changes stream will wait and poll for changes. Otherwise,
    /// the stream will return all changes until now and then close.
    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite;
        let timeout = match infinite {
            true => COUCH_MAX_TIMEOUT.to_string(),
            false => 0.to_string(),
        };
        self.params.insert("timeout".to_string(), timeout);
    }

    /// Get the last retrieved seq.
    pub fn last_seq(&self) -> &Option<serde_json::Value> {
        &self.last_seq
    }

    /// Whether this stream is running in infinite mode.
    pub fn infinite(&self) -> bool {
        self.infinite
    }
}

async fn get_changes(client: Client, database: String, params: HashMap<String, String>) -> CouchResult<Response> {
    let path = format!("{}/_changes", database);
    let res = client.req(Method::GET, &path, Some(&params)).send().await?;
    Ok(res)
}

impl Stream for ChangesStream {
    type Item = CouchResult<ChangeEvent>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            self.state = match self.state {
                ChangesStreamState::Idle => {
                    let mut params = self.params.clone();
                    if let Some(seq) = &self.last_seq {
                        params.insert("since".to_string(), seq.to_string());
                    }
                    let fut = get_changes(self.client.clone(), self.database.clone(), params);
                    ChangesStreamState::Requesting(Box::pin(fut))
                }
                ChangesStreamState::Requesting(ref mut fut) => match ready!(fut.poll_unpin(cx)) {
                    Err(err) => return Poll::Ready(Some(Err(err))),
                    Ok(res) => match res.status().is_success() {
                        true => {
                            let stream = res
                                .bytes_stream()
                                .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
                            let reader = StreamReader::new(stream);
                            let lines = Box::pin(LinesStream::new(reader.lines()));
                            ChangesStreamState::Reading(lines)
                        }
                        false => {
                            return Poll::Ready(Some(Err(CouchError::new(
                                res.status().canonical_reason().unwrap().to_string(),
                                res.status(),
                            ))))
                        }
                    },
                },
                ChangesStreamState::Reading(ref mut lines) => {
                    let line = ready!(lines.poll_next_unpin(cx));
                    match line {
                        None => ChangesStreamState::Idle,
                        Some(Err(err)) => {
                            let inner = err.get_ref().and_then(|err| err.downcast_ref::<reqwest::Error>());
                            match inner {
                                Some(reqwest_err) if reqwest_err.is_timeout() && self.infinite => {
                                    ChangesStreamState::Idle
                                }
                                Some(reqwest_err) => {
                                    return Poll::Ready(Some(Err(CouchError::new(
                                        reqwest_err.to_string(),
                                        reqwest_err.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                                    ))));
                                }
                                _ => {
                                    return Poll::Ready(Some(Err(CouchError::new(
                                        format!("{}", err),
                                        StatusCode::from_u16(500).unwrap(),
                                    ))));
                                }
                            }
                        }
                        Some(Ok(line)) if line.is_empty() => continue,
                        Some(Ok(line)) => match serde_json::from_str::<Event>(&line) {
                            Ok(Event::Change(event)) => {
                                self.last_seq = Some(event.seq.clone());
                                return Poll::Ready(Some(Ok(event)));
                            }
                            Ok(Event::Finished(event)) => {
                                self.last_seq = Some(event.last_seq.clone());
                                if !self.infinite {
                                    return Poll::Ready(None);
                                }
                                ChangesStreamState::Idle
                            }
                            Err(e) => {
                                return Poll::Ready(Some(Err(e.into())));
                            }
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use futures_util::StreamExt;
    use serde_json::{json, Value};
    #[tokio::test]
    async fn should_get_changes() {
        let client = Client::new_local_test().unwrap();
        let db = client.db("should_get_changes").await.unwrap();
        let mut changes = db.changes(None);
        changes.set_infinite(true);
        let t = tokio::spawn({
            let db = db.clone();
            async move {
                let mut docs: Vec<Value> = (0..10)
                    .map(|idx| {
                        json!({
                            "_id": format!("test_{}", idx),
                            "count": idx,
                        })
                    })
                    .collect();

                db.bulk_docs(&mut docs).await.expect("should insert 10 documents");
            }
        });

        let mut collected_changes = vec![];
        while let Some(change) = changes.next().await {
            collected_changes.push(change);
            if collected_changes.len() == 10 {
                break;
            }
        }
        assert!(collected_changes.len() == 10);
        t.await.unwrap();
    }
}
