pub mod ovnnb;
pub mod vswitch;
use std::cell::{Ref, RefCell};
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

pub mod request;
use request::{Method, Request};

pub mod response;
use response::Response;

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

pub trait Connection {
    fn disconnect(&self) -> Result<(), Error>;
    fn send(&self, payload: &[u8]) -> Result<(), Error>;
    fn recv(&self) -> Result<Vec<u8>, Error>;
}

pub struct UnixConnection {
    stream: Option<RefCell<UnixStream>>,
}

impl UnixConnection {
    pub fn connect(path: &Path) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let stream = UnixStream::connect(path)?;
        Ok(Self {
            stream: Some(RefCell::new(stream)),
        })
    }
}

impl Connection for UnixConnection {
    fn disconnect(&self) -> Result<(), Error> {
        if let Some(s) = &self.stream {
            s.borrow().shutdown(Shutdown::Both).unwrap();
            Ok(())
        } else {
            Err(Error::NotConnected)
        }
    }
    fn send(&self, payload: &[u8]) -> Result<(), Error> {
        if let Some(s) = &self.stream {
            s.borrow_mut().write_all(payload).unwrap();
            assert!(s.borrow_mut().write(b"\0").unwrap() == 1);
            Ok(())
        } else {
            Err(Error::NotConnected)
        }
    }

    fn recv(&self) -> Result<Vec<u8>, Error> {
        if let Some(s) = &self.stream {
            let _ = s.borrow_mut().flush();
            let mut data = vec![];

            s.borrow_mut().read_to_end(&mut data).unwrap();

            println!("response: {}", std::str::from_utf8(&data).unwrap());
            Ok(data)
        } else {
            Err(Error::NotConnected)
        }
    }
}

pub struct Client<T>
where
    T: Connection,
{
    conn: RefCell<T>,
}

impl<T> Client<T>
where
    T: Connection,
{
    pub fn disconnect(&self) -> Result<(), Error> {
        self.conn.borrow_mut().disconnect()
    }

    pub fn conn(&self) -> Ref<T> {
        self.conn.borrow()
    }

    pub fn execute<R>(&self, method: &Method) -> Result<Response<R>, Error>
    where
        R: DeserializeOwned,
    {
        let request = Request {
            id: "12345".to_string(),
            method,
        };
        let payload = serde_json::to_vec(&request).unwrap();
        self.conn().send(&payload).unwrap();

        let data = self.conn().recv().unwrap();

        let res: Response<R> = serde_json::from_slice(&data).unwrap();
        Ok(res)
    }
}

pub fn connect_unix(socket: &Path) -> Result<Client<UnixConnection>, Error> {
    let conn = UnixConnection::connect(socket)?;
    Ok(Client {
        conn: RefCell::new(conn),
    })
}
