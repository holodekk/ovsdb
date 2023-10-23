use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::result::Result;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json;

mod ovsdb;

use ovsdb::{
    OVSBridge, OVSDBError, OVSDBListResult, OVSDBMethod, OVSDBOperation, OVSDBRequest,
    OVSDBTransactParams,
};

#[derive(Debug, Deserialize)]
struct ListBridgesResponse {
    result: Vec<OVSDBListResult<OVSBridge>>,
}

pub struct OVSDBClient {
    stream: Option<UnixStream>,
}

impl OVSDBClient {
    pub fn connect_unix(socket: &Path) -> Result<OVSDBClient, OVSDBError> {
        let stream = UnixStream::connect(socket)?;
        Ok(OVSDBClient {
            stream: Some(stream),
        })
    }

    pub fn disconnect(&mut self) -> Result<(), OVSDBError> {
        if let Some(ref mut stream) = self.stream {
            stream.shutdown(Shutdown::Both).unwrap();
            Ok(())
        } else {
            Err(OVSDBError::NotConnected)
        }
    }

    // fn send(&mut self, payload: OVSDBRequest) -> Result<Vec<u8>, OVSDBError> {
    //     if let Some(ref mut stream) = self.stream {
    //         serde_json::to_writer(&*stream, &payload).unwrap();
    //         stream.write(b"\0").unwrap();
    //         let _ = stream.flush();
    //         let mut s = vec![];

    //         stream.read_to_end(&mut s).unwrap();
    //         Ok(s)
    //     } else {
    //         Err(OVSDBError::NotConnected)
    //     }
    // }

    fn send<'b, T>(&mut self, payload: OVSDBRequest) -> Result<T, OVSDBError>
    where
        T: DeserializeOwned,
    {
        if let Some(ref mut stream) = self.stream {
            serde_json::to_writer(&*stream, &payload).unwrap();
            stream.write(b"\0").unwrap();
            let _ = stream.flush();
            let mut s = vec![];

            stream.read_to_end(&mut s).unwrap();
            let result: T = serde_json::from_slice(&s).unwrap();
            Ok(result)
        } else {
            Err(OVSDBError::NotConnected)
        }
    }

    pub fn list_bridges(&mut self) -> Result<Option<Vec<OVSBridge>>, OVSDBError> {
        let request = OVSDBRequest {
            id: "123456".to_string(),
            method: OVSDBMethod::Transact(OVSDBTransactParams {
                database: "Open_vSwitch".to_string(),
                operations: vec![OVSDBOperation::Select {
                    table: "Bridge".to_string(),
                    clauses: vec![],
                }],
            }),
        };

        let res: ListBridgesResponse = self.send(request).unwrap();

        // println!("response: {}", std::str::from_utf8(&response).unwrap());

        // let res: ListBridgesResponse = serde_json::from_slice(&response).unwrap();

        match res.result.into_iter().nth(0) {
            Some(result) => Ok(Some(result.rows)),
            None => Ok(None),
        }
    }
}
