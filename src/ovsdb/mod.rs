use std::cell::{Ref, RefCell};
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ovs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Bridge {
    pub _uuid: ovs::Atom,
    pub name: String,
    pub datapath_type: String,
    pub fail_mode: String,
    pub ports: ovs::Atom,
}

impl ovs::Entity for Bridge {
    fn table() -> &'static str {
        "Bridge"
    }
}

#[derive(Deserialize, Serialize)]
pub struct Port {
    _uuid: ovs::Atom,
    name: String,
    interfaces: Vec<Uuid>,
}

pub enum Tables {
    Bridge,
}

pub struct Client<T>
where
    T: ovs::Connection,
{
    conn: RefCell<T>,
}

impl<T> ovs::Client<T> for Client<T>
where
    T: ovs::Connection,
{
    fn disconnect(&self) -> Result<(), ovs::Error> {
        self.conn.borrow_mut().disconnect()
    }

    fn conn(&self) -> Ref<T> {
        self.conn.borrow()
    }

    fn database(&self) -> &str {
        "Open_vSwitch"
    }

    // pub fn list<'a, R>(&self) -> Result<Option<Vec<R>>, ovs::Error>
    // where
    //     R: ovs::Listable + DeserializeOwned,
    // {
    //     let request = R::builder().build();

    //     let conn = self.conn.borrow();
    //     conn.send(request).unwrap();
    //     let res: ovs::ListResponse<R> = serde_json::from_slice(&conn.recv().unwrap()).unwrap();
    //     match res.result.into_iter().nth(0) {
    //         Some(result) => Ok(Some(result.rows)),
    //         None => Ok(None),
    //     }
    // }
}

impl Client<ovs::UnixConnection> {
    pub fn connect_unix() -> Result<Self, ovs::Error> {
        let conn = ovs::UnixConnection::connect(Path::new("/var/run/openvswitch/db.sock"))?;
        Ok(Self {
            conn: RefCell::new(conn),
        })
    }
}
