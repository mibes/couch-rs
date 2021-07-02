use crate::client::Client;
use futures_util::{ready, FutureExt, StreamExt, TryStreamExt};
use futures_core::{Future, Stream};
use reqwest::StatusCode;
use reqwest::{Method, Response};
use std::io;
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;
use tokio_util::io::StreamReader;

use crate::error::{CouchError, CouchResult};
use crate::types::changes::{ChangeEvent, Event};

pub struct ChangesStream {
    last_seq: Option<String>,
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
    pub fn new(client: Client, database: String, last_seq: Option<String>) -> Self {
        let mut params = HashMap::new();
        params.insert("feed".to_string(), "continuous".to_string());
        params.insert("timeout".to_string(), "0".to_string());
        params.insert("include_docs".to_string(), "true".to_string());
        Self::with_params(client, database, last_seq, params)
    }

    pub fn with_params(
        client: Client,
        database: String,
        last_seq: Option<String>,
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

    pub fn set_last_seq(&mut self, last_seq: Option<String>) {
        self.last_seq = last_seq;
    }

    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite;
        let timeout = match infinite {
            true => "60000".to_string(),
            false => "0".to_string(),
        };
        self.params.insert("timeout".to_string(), timeout);
    }

    pub fn last_seq(&self) -> &Option<String> {
        &self.last_seq
    }

    pub fn infinite(&self) -> bool {
        self.infinite
    }
}

async fn get_changes(client: Client, database: String, params: HashMap<String, String>) -> CouchResult<Response> {
    let path = format!("{}/_changes", database);
    let res = client.req(Method::GET, path, Some(params)).send().await?;
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
                        params.insert("since".to_string(), seq.clone());
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
                                .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{}", err)));
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
                        Some(Err(e)) => {
                            return Poll::Ready(Some(Err(CouchError::new(
                                format!("{}", e),
                                StatusCode::from_u16(500).unwrap(),
                            ))))
                        }
                        Some(Ok(line)) if line.is_empty() => continue,
                        Some(Ok(line)) => match serde_json::from_str::<Event>(&line) {
                            Ok(Event::Change(event)) => {
                                self.last_seq = Some(event.seq.clone());
                                // eprintln!("event {:?}", event);
                                return Poll::Ready(Some(Ok(event)));
                            }
                            Ok(Event::Finished(event)) => {
                                self.last_seq = Some(event.last_seq.clone());
                                if !self.infinite {
                                    return Poll::Ready(None);
                                }
                                // eprintln!("event {:?}", event);
                                ChangesStreamState::Idle
                            }
                            Err(e) => {
                                // eprintln!("Decoding error {} on line {}", e, line);
                                return Poll::Ready(Some(Err(e.into())));
                            }
                        },
                    }
                }
            }
        }
    }
}
