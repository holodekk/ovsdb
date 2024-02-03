use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;
use uuid::Uuid;

pub mod rpc;

pub mod request;

pub mod response;

pub mod schema;

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
    // fn load_table<'a>(data: &'a [u8]) -> Result<Option<Vec<Self>>, Error>
    // where
    //     Self: Sized;
}

// pub trait Connection {
//     fn disconnect(&self) -> Result<(), Error>;
//     fn send(&self, payload: &[u8]) -> Result<(), Error>;
//     fn recv(&self) -> Result<Option<Vec<u8>>, Error>;
// }

// pub struct UnixConnection {
//     stream: Option<RefCell<UnixStream>>,
// }

// impl UnixConnection {
//     pub fn connect(path: &Path) -> Result<Self, Error>
//     where
//         Self: Sized,
//     {
//         let stream = UnixStream::connect(path)?;
//         // stream
//         //     .set_read_timeout(Some(Duration::new(1, 0)))
//         //     .expect("Couldn't set read timeout");
//         Ok(Self {
//             stream: Some(RefCell::new(stream)),
//         })
//     }
// }

// impl Connection for UnixConnection {
//     fn disconnect(&self) -> Result<(), Error> {
//         if let Some(s) = &self.stream {
//             s.borrow().shutdown(Shutdown::Both).unwrap();
//             Ok(())
//         } else {
//             Err(Error::NotConnected)
//         }
//     }
//     fn send(&self, payload: &[u8]) -> Result<(), Error> {
//         if let Some(s) = &self.stream {
//             s.borrow_mut().write_all(payload).unwrap();
//             println!("Sent {:?}", std::str::from_utf8(payload).unwrap());

//             Ok(())
//         } else {
//             Err(Error::NotConnected)
//         }
//     }

//     fn recv(&self) -> Result<Option<Vec<u8>>, Error> {
//         if let Some(s) = &self.stream {
//             let _ = s.borrow_mut().flush();
//             let mut data = vec![];

//             let bytes_read = s.borrow_mut().read(&mut data).unwrap();

//             if bytes_read > 0 {
//                 println!("read: {}", std::str::from_utf8(&data).unwrap());
//                 Ok(Some(data))
//             } else {
//                 println!("empty read");
//                 Ok(None)
//             }
//         } else {
//             Err(Error::NotConnected)
//         }
//     }
// }

pub struct Client {
    rpc: rpc::Rpc,
}

impl Client {
    pub fn new(rpc: rpc::Rpc) -> Self {
        Self { rpc }
    }

    pub async fn connect_unix(socket: &Path) -> Result<Client, Error> {
        let stream = UnixStream::connect(socket).await?;
        let rpc = rpc::Rpc::start(stream).await.unwrap();
        Ok(Client::new(rpc))
        // conn: RefCell::new(conn),
        // })
    }

    pub async fn echo(&mut self, params: Vec<String>) -> Result<Vec<String>, Error> {
        match self.rpc.execute(rpc::Method::Echo, Some(params)).await {
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

    pub async fn get_schema(&mut self, database: &str) -> Result<schema::Schema, Error> {
        match self
            .rpc
            .execute(rpc::Method::GetSchema, Some(vec![database]))
            .await
        {
            Ok(rx) => match rx.await {
                Ok(res) => {
                    let s: schema::Schema = serde_json::from_value(res.result).unwrap();
                    Ok(s)
                }
                Err(_err) => Err(Error::Unknown),
            },
            Err(err) => Err(err),
        }
    }
    // pub fn disconnect(&self) -> Result<(), Error> {
    //     self.conn.borrow_mut().disconnect()
    // }

    // pub fn conn(&self) -> Ref<T> {
    //     self.conn.borrow()
    // }

    // pub fn execute<R>(&self, method: &Method) -> Result<Response<R>, Error>
    // where
    //     R: DeserializeOwned,
    // {
    //     let mut buffer = Buffer::new();
    //     let request = Request {
    //         id: "12345".to_string(),
    //         method,
    //     };
    //     let payload = serde_json::to_vec(&request).unwrap();
    //     self.conn().send(&payload).unwrap();

    //     let data = self.conn().recv().unwrap();

    //     match buffer.process(&data) {
    //         Some(messages) => {
    //             let res: Response<R> = serde_json::from_slice(&messages[0]).unwrap();
    //             Ok(res)
    //         }
    //         None => panic!("No response received"),
    //     }
    // }
}

// pub fn connect_unix(socket: &Path) -> Result<Client<UnixConnection>, Error> {
//     let conn = UnixConnection::connect(socket)?;
//     Ok(Client::new(conn))
//     // conn: RefCell::new(conn),
//     // })
// }
