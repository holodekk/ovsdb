use std::cell::{Ref, RefCell};
use std::path::Path;

use serde::{Deserialize, Serialize};
// use uuid::Uuid;

use crate::ovs;

#[derive(Debug, Deserialize, Serialize)]
pub struct LogicalSwitch {
    pub _uuid: ovs::Atom,
    pub name: String,
}

impl ovs::Entity for LogicalSwitch {
    fn table() -> &'static str {
        "Logical_Switch"
    }
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
        "OVN_Northbound"
    }
}

impl Client<ovs::UnixConnection> {
    pub fn connect_unix() -> Result<Self, ovs::Error> {
        let conn = ovs::UnixConnection::connect(Path::new("/var/run/ovn/ovnnb_db.sock"))?;
        Ok(Self {
            conn: RefCell::new(conn),
        })
    }
}
