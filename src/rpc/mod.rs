use std::collections::HashMap;

use futures::stream::StreamExt;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_json::Value;
use tokio::{
    io::AsyncWriteExt,
    net::UnixStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_util::codec::FramedRead;

mod codec;
pub use codec::JsonCodec;

#[derive(Debug)]
pub enum Method {
    Echo,
    ListDatabases,
    GetSchema,
    // Transact,
    // Cancel,
    // Monitor,
    // Update,
    // MonitorCancel,
    // Lock,
    // Steal,
    // Unlock,
    // Locked,
    // Stolen,
}

impl Method {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Echo => "echo",
            Self::ListDatabases => "list_dbs",
            Self::GetSchema => "get_schema",
        }
    }
}

pub struct Request {
    pub id: uuid::Uuid,
    pub method: Method,
    pub params: Value,
}

impl Request {
    pub fn new(method: Method, params: Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            method,
            params,
        }
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("method", self.method.name())?;
        map.serialize_entry("params", &self.params)?;
        map.end()
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub result: Value,
    pub error: Option<String>,
    pub id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListResult<T> {
    pub rows: Vec<T>,
}

pub enum Event {}

pub struct Rpc {
    // pub events: mpsc::Receiver<Event>,
    pub sender: Option<mpsc::Sender<Payload>>,
    // pub receiver: mpsc::Receiver<Response>,
    pub handle: JoinHandle<()>,
}

impl Rpc {
    pub async fn stop(mut self) {
        if let Some(sender) = self.sender.take() {
            drop(sender);
        }
        if let Err(err) = self.handle.await {
            println!("Error encountered waiting for Rpc shutdown: {err}");
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown error")]
    Unknown,
}

pub struct Payload {
    tx: oneshot::Sender<Response>,
    request: Request,
}

impl Rpc {
    pub async fn start(conn: UnixStream) -> Result<Rpc, Error> {
        let (requests_tx, requests_rx) = mpsc::channel(32);

        let handle = {
            println!("Spawning rpc_main()");
            tokio::spawn(async move { rpc_main(requests_rx, conn).await.unwrap() })
        };

        Ok(Rpc {
            sender: Some(requests_tx),
            // receiver: responses_rx,
            handle,
        })
    }

    pub async fn execute<P>(
        &mut self,
        method: Method,
        params: Option<P>,
    ) -> Result<oneshot::Receiver<Response>, crate::Error>
    where
        P: Serialize,
    {
        println!("Rpc::execute({:?})", method);
        let (tx, rx) = oneshot::channel();
        let p = match params {
            Some(v) => serde_json::to_value(v).unwrap(),
            None => serde_json::to_value::<Vec<i32>>(vec![]).unwrap(),
        };
        let request = Request::new(method, p);

        if let Some(s) = &self.sender {
            s.send(Payload { tx, request }).await.unwrap();
        }

        Ok(rx)
    }
}

async fn rpc_main(
    mut requests: mpsc::Receiver<Payload>,
    mut conn: UnixStream,
) -> Result<(), std::io::Error> {
    let (read, mut write) = conn.split();

    let mut channels: HashMap<uuid::Uuid, oneshot::Sender<Response>> = HashMap::new();

    // let _ = conn.shutdown().await;
    let mut json = FramedRead::new(read, JsonCodec::new());

    loop {
        tokio::select! {
            Some(payload) = requests.recv() => {
                channels.insert(payload.request.id, payload.tx);
                let payload = serde_json::to_vec(&payload.request)?;
                write.write_all(&payload).await?;
            }
            Some(data) = json.next() => {
                let res: Response = serde_json::from_value(data.unwrap())?;
                if let Some(tx) = channels.remove(&res.id) {
                    tx.send(res).unwrap();
                }
            }
            else => {
                println!("All senders closed.  Exiting.");
                break;
            }
        }
    }

    Ok(())
}
