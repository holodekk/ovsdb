use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use uuid::Uuid;

mod connection;
pub use connection::*;

pub mod codec;
pub mod request;
pub mod response;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown Error")]
    Unknown,
    #[error("Not Connected")]
    NotConnected,
    #[error("Unexpected IO Error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[allow(non_camel_case_types)]
pub enum Atom {
    map(String, Vec<(String, String)>),
    set(String, Vec<Atom>),
    uuid(String, Uuid),
}

pub trait Entity {
    fn table_name() -> &'static str;
}

pub struct Payload {
    tx: oneshot::Sender<response::Response>,
    request: request::Request,
}

pub struct Client {
    pub sender: Option<mpsc::Sender<Payload>>,
    pub handle: JoinHandle<()>,
}

impl Client {
    pub fn new(sender: mpsc::Sender<Payload>, handle: JoinHandle<()>) -> Self {
        Self {
            sender: Some(sender),
            handle,
        }
    }

    pub async fn start<T>(conn: T) -> Result<Self, Error>
    where
        T: Connection + Send + 'static,
    {
        let (requests_tx, requests_rx) = mpsc::channel(32);

        let handle = {
            println!("Spawning client_main()");
            tokio::spawn(async move { client_main(requests_rx, conn).await.unwrap() })
        };

        Ok(Client::new(requests_tx, handle))
    }

    pub async fn connect_unix(socket: &Path) -> Result<Client, Error> {
        let stream = UnixStream::connect(socket).await?;
        let conn = UnixConnection::new(stream);
        // let rpc = rpc::Rpc::start(conn).await.unwrap();
        Client::start(conn).await
    }

    pub async fn execute<P>(
        &mut self,
        method: request::Method,
        params: Option<P>,
    ) -> Result<oneshot::Receiver<response::Response>, crate::Error>
    where
        P: Serialize,
    {
        println!("Client::execute({:?})", method);
        let (tx, rx) = oneshot::channel();
        let p = match params {
            Some(v) => serde_json::to_value(v).unwrap(),
            None => serde_json::to_value::<Vec<i32>>(vec![]).unwrap(),
        };
        let request = request::Request::new(method, p);

        if let Some(s) = &self.sender {
            s.send(Payload { tx, request }).await.unwrap();
        }

        Ok(rx)
    }

    pub async fn echo(&mut self, params: Vec<String>) -> Result<Vec<String>, Error> {
        match self.execute(request::Method::Echo, Some(params)).await {
            Ok(rx) => match rx.await {
                Ok(res) => {
                    let p: Vec<String> = serde_json::from_value(res.result).unwrap();
                    Ok(p)
                }
                Err(_err) => Err(Error::Unknown),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn get_schema(&mut self, database: &str) -> Result<crate::schema::Schema, Error> {
        match self
            .execute(request::Method::GetSchema, Some(vec![database]))
            .await
        {
            Ok(rx) => match rx.await {
                Ok(res) => {
                    let s: crate::schema::Schema = serde_json::from_value(res.result).unwrap();
                    Ok(s)
                }
                Err(_err) => Err(Error::Unknown),
            },
            Err(err) => Err(err),
        }
    }
}

async fn client_main<T>(
    mut requests: mpsc::Receiver<Payload>,
    mut conn: T,
) -> Result<(), std::io::Error>
where
    T: Connection,
{
    let mut channels: HashMap<uuid::Uuid, oneshot::Sender<response::Response>> = HashMap::new();

    loop {
        tokio::select! {
            Some(payload) = requests.recv() => {
                channels.insert(payload.request.id, payload.tx);
                conn.send(payload.request).await?;
            }
            Some(data) = conn.next() => {
                let res: response::Response = serde_json::from_value(data.unwrap())?;
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
