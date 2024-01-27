use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;

use crate::ovs::{Error, Request};

pub trait Connection {
    fn disconnect(&self) -> Result<(), Error>;
    fn send(&self, payload: Request) -> Result<(), Error>;
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
    fn send(&self, payload: Request) -> Result<(), Error> {
        if let Some(s) = &self.stream {
            serde_json::to_writer(&*s.borrow(), &payload).unwrap();
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
